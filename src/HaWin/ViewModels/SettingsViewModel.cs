using System.ComponentModel;
using System.Reflection;
using System.Runtime.CompilerServices;
using HaWin.Models;
using HaWin.Utilities;

namespace HaWin.ViewModels;

public class SettingsViewModel : INotifyPropertyChanged
{
    private string _brokerHost = "";
    private int _brokerPort = 1883;
    private string _username = "";
    private string _password = "";
    private bool _useTls;
    private bool _autoStart;
    private string _clientId = "";
    private bool _isConnected;

    public string MachineName { get; } = Environment.MachineName;
    public string DeviceId { get; } = TopicHelper.GetDeviceId();
    public string NotifyTopic => $"ha-win/{DeviceId}/notify";
    public string AppVersion { get; } = GetAppVersion();

    public string BrokerHost
    {
        get => _brokerHost;
        set => SetField(ref _brokerHost, value);
    }

    public int BrokerPort
    {
        get => _brokerPort;
        set => SetField(ref _brokerPort, value);
    }

    public string Username
    {
        get => _username;
        set => SetField(ref _username, value);
    }

    public string Password
    {
        get => _password;
        set => SetField(ref _password, value);
    }

    public bool UseTls
    {
        get => _useTls;
        set => SetField(ref _useTls, value);
    }

    public bool AutoStart
    {
        get => _autoStart;
        set => SetField(ref _autoStart, value);
    }

    public string ClientId
    {
        get => _clientId;
        set => SetField(ref _clientId, value);
    }

    public bool IsConnected
    {
        get => _isConnected;
        set => SetField(ref _isConnected, value);
    }

    public void LoadFrom(AppSettings settings)
    {
        BrokerHost = settings.BrokerHost;
        BrokerPort = settings.BrokerPort;
        Username = settings.Username;
        Password = settings.Password;
        UseTls = settings.UseTls;
        AutoStart = settings.AutoStart;
        ClientId = settings.ClientId;
    }

    public AppSettings ToSettings()
    {
        return new AppSettings
        {
            BrokerHost = BrokerHost,
            BrokerPort = BrokerPort,
            Username = Username,
            Password = Password,
            UseTls = UseTls,
            AutoStart = AutoStart,
            ClientId = ClientId
        };
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    private void SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (Equals(field, value))
        {
            return;
        }

        field = value;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }

    private static string GetAppVersion()
    {
        return Assembly.GetExecutingAssembly()
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
            ?? Assembly.GetExecutingAssembly().GetName().Version?.ToString()
            ?? "unknown";
    }
}
