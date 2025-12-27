using System.Text;
using System.Text.Json;
using HaWin.Models;
using HaWin.Utilities;
using MQTTnet;
using MQTTnet.Client;

namespace HaWin.Services;

public class MqttService : IDisposable
{
    private readonly NotificationService _notificationService;
    private readonly HomeAssistantDiscovery _discovery = new();
    private string _topicNamespace = TopicHelper.GetDeviceId();
    private readonly string _machineName = Environment.MachineName;

    private IMqttClient? _client;
    private MqttClientOptions? _options;
    private AppSettings? _settings;
    private bool _disposed;
    private bool _isConnected;

    public event EventHandler<bool>? ConnectionChanged;
    public bool IsConnected => _isConnected;

    public MqttService(NotificationService notificationService)
    {
        _notificationService = notificationService;
    }

    public async Task ApplySettingsAsync(AppSettings settings)
    {
        _settings = settings;
        _topicNamespace = string.IsNullOrWhiteSpace(settings.Namespace)
            ? TopicHelper.GetDeviceId()
            : TopicHelper.SanitizeNamespace(settings.Namespace);
        await ConnectAsync();
    }

    public async Task<bool> TestConnectionAsync(AppSettings settings, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(settings.BrokerHost))
        {
            return false;
        }

        var factory = new MqttFactory();
        using var client = factory.CreateMqttClient();

        var clientId = string.IsNullOrWhiteSpace(settings.ClientId)
            ? $"HaWin-Test-{_topicNamespace}"
            : $"{settings.ClientId}-test";

        var optionsBuilder = new MqttClientOptionsBuilder()
            .WithClientId(clientId)
            .WithTcpServer(settings.BrokerHost, settings.BrokerPort)
            .WithCleanSession();

        if (!string.IsNullOrWhiteSpace(settings.Username))
        {
            optionsBuilder = optionsBuilder.WithCredentials(settings.Username, settings.Password);
        }

        if (settings.UseTls)
        {
            optionsBuilder = optionsBuilder.WithTlsOptions(_ => { });
        }

        var options = optionsBuilder.Build();

        try
        {
            await client.ConnectAsync(options, cancellationToken);
            var disconnectOptions = new MqttClientDisconnectOptionsBuilder()
                .WithReason(MqttClientDisconnectOptionsReason.NormalDisconnection)
                .Build();
            await client.DisconnectAsync(disconnectOptions, cancellationToken);
            return true;
        }
        catch
        {
            return false;
        }
    }

    private async Task ConnectAsync()
    {
        if (_settings == null || string.IsNullOrWhiteSpace(_settings.BrokerHost))
        {
            return;
        }

        if (_client != null)
        {
            try
            {
                await _client.DisconnectAsync();
            }
            catch
            {
            }
        }

        var factory = new MqttFactory();
        _client = factory.CreateMqttClient();

        _client.ApplicationMessageReceivedAsync += OnMessageReceived;
        _client.ConnectedAsync += OnConnected;
        _client.DisconnectedAsync += OnDisconnected;

        var clientId = string.IsNullOrWhiteSpace(_settings.ClientId)
            ? $"HaWin-{_topicNamespace}"
            : _settings.ClientId;

        var optionsBuilder = new MqttClientOptionsBuilder()
            .WithClientId(clientId)
            .WithTcpServer(_settings.BrokerHost, _settings.BrokerPort)
            .WithCleanSession();

        if (!string.IsNullOrWhiteSpace(_settings.Username))
        {
            optionsBuilder = optionsBuilder.WithCredentials(_settings.Username, _settings.Password);
        }

        if (_settings.UseTls)
        {
            optionsBuilder = optionsBuilder.WithTlsOptions(_ => { });
        }

        optionsBuilder = optionsBuilder
            .WithWillTopic(_discovery.StatusTopic(_topicNamespace))
            .WithWillPayload("offline")
            .WithWillRetain(true);

        _options = optionsBuilder.Build();

        try
        {
            await _client.ConnectAsync(_options, CancellationToken.None);
        }
        catch
        {
            SetConnectionState(false);
        }
    }

    private async Task OnConnected(MqttClientConnectedEventArgs args)
    {
        SetConnectionState(true);

        await PublishStatusAsync("online");
        await PublishDiscoveryAsync();
        await SubscribeAsync();
    }

    private async Task OnDisconnected(MqttClientDisconnectedEventArgs args)
    {
        SetConnectionState(false);

        if (_disposed || _options == null || _client == null)
        {
            return;
        }

        await Task.Delay(TimeSpan.FromSeconds(5));

        try
        {
            await _client.ConnectAsync(_options, CancellationToken.None);
        }
        catch
        {
        }
    }

    private async Task SubscribeAsync()
    {
        if (_client == null)
        {
            return;
        }

        var actionTopic = $"{_discovery.BaseTopic(_topicNamespace)}/+/set";
        var notifyTopic = _discovery.NotifyCommandTopic(_topicNamespace);

        await _client.SubscribeAsync(actionTopic);
        await _client.SubscribeAsync(notifyTopic);
    }

    private async Task PublishDiscoveryAsync()
    {
        if (_client == null)
        {
            return;
        }

        await PublishRetainedAsync(
            _discovery.ButtonConfigTopic(_topicNamespace, "restart"),
            _discovery.BuildButtonConfig(_topicNamespace, _machineName, "restart", "Restart PC", "mdi:restart"));

        await PublishRetainedAsync(
            _discovery.ButtonConfigTopic(_topicNamespace, "shutdown"),
            _discovery.BuildButtonConfig(_topicNamespace, _machineName, "shutdown", "Shutdown PC", "mdi:power"));

        await PublishRetainedAsync(
            _discovery.ButtonConfigTopic(_topicNamespace, "standby"),
            _discovery.BuildButtonConfig(_topicNamespace, _machineName, "standby", "Standby PC", "mdi:sleep"));

        await PublishRetainedAsync(
            _discovery.NotifyConfigTopic(_topicNamespace),
            _discovery.BuildNotifyConfig(_topicNamespace, _machineName));
    }

    private async Task PublishStatusAsync(string status)
    {
        await PublishRetainedAsync(_discovery.StatusTopic(_topicNamespace), status);
    }

    private async Task PublishRetainedAsync(string topic, string payload)
    {
        if (_client == null)
        {
            return;
        }

        var message = new MqttApplicationMessageBuilder()
            .WithTopic(topic)
            .WithPayload(payload)
            .WithRetainFlag(true)
            .Build();

        await _client.PublishAsync(message, CancellationToken.None);
    }

    private Task OnMessageReceived(MqttApplicationMessageReceivedEventArgs args)
    {
        var topic = args.ApplicationMessage.Topic;
        var payload = Encoding.UTF8.GetString(args.ApplicationMessage.PayloadSegment);

        if (topic == _discovery.NotifyCommandTopic(_topicNamespace))
        {
            HandleNotification(payload);
            return Task.CompletedTask;
        }

        if (!topic.StartsWith(_discovery.BaseTopic(_topicNamespace) + "/", StringComparison.OrdinalIgnoreCase))
        {
            return Task.CompletedTask;
        }

        if (payload != "PRESS")
        {
            return Task.CompletedTask;
        }

        if (topic.EndsWith("/restart/set", StringComparison.OrdinalIgnoreCase))
        {
            DeviceActions.Restart();
        }
        else if (topic.EndsWith("/shutdown/set", StringComparison.OrdinalIgnoreCase))
        {
            DeviceActions.Shutdown();
        }
        else if (topic.EndsWith("/standby/set", StringComparison.OrdinalIgnoreCase))
        {
            DeviceActions.Standby();
        }

        return Task.CompletedTask;
    }

    private void HandleNotification(string payload)
    {
        var title = "Home Assistant";
        var message = payload;

        try
        {
            using var doc = JsonDocument.Parse(payload);
            if (doc.RootElement.ValueKind == JsonValueKind.Object)
            {
                if (doc.RootElement.TryGetProperty("title", out var titleProp))
                {
                    title = titleProp.GetString() ?? title;
                }

                if (doc.RootElement.TryGetProperty("message", out var messageProp))
                {
                    message = messageProp.GetString() ?? message;
                }
            }
        }
        catch
        {
        }

        if (string.IsNullOrWhiteSpace(message))
        {
            message = payload;
        }

        _notificationService.ShowNotification(title, message);
    }

    public void Dispose()
    {
        _disposed = true;
        SetConnectionState(false);
        _client?.Dispose();
    }

    private void SetConnectionState(bool isConnected)
    {
        _isConnected = isConnected;
        ConnectionChanged?.Invoke(this, isConnected);
    }
}
