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
    private string _namespace = TopicHelper.GetDeviceId();
    private bool _useTls;
    private bool _autoStart;
    private bool _autoCheckUpdates;
    private string _clientId = "";
    private bool _showWindowOnStartup = true;
    private bool _isConnected;
    private bool _isDirty;
    private AppSettings _baseline = new();

    public string MachineName { get; } = Environment.MachineName;
    public string DeviceId => TopicHelper.SanitizeNamespace(Namespace);
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

    public string Namespace
    {
        get => _namespace;
        set
        {
            if (SetField(ref _namespace, value))
            {
                PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(NotifyTopic)));
            }
        }
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

    public bool AutoCheckUpdates
    {
        get => _autoCheckUpdates;
        set => SetField(ref _autoCheckUpdates, value);
    }

    public string ClientId
    {
        get => _clientId;
        set => SetField(ref _clientId, value);
    }

    public bool ShowWindowOnStartup
    {
        get => _showWindowOnStartup;
        set => SetField(ref _showWindowOnStartup, value);
    }

    public bool IsConnected
    {
        get => _isConnected;
        set => SetField(ref _isConnected, value);
    }

    public bool IsDirty
    {
        get => _isDirty;
        private set => SetField(ref _isDirty, value);
    }

    public void LoadFrom(AppSettings settings)
    {
        BrokerHost = settings.BrokerHost;
        BrokerPort = settings.BrokerPort;
        Username = settings.Username;
        Password = settings.Password;
        Namespace = string.IsNullOrWhiteSpace(settings.Namespace)
            ? TopicHelper.GetDeviceId()
            : settings.Namespace;
        UseTls = settings.UseTls;
        AutoStart = settings.AutoStart;
        AutoCheckUpdates = settings.AutoCheckUpdates;
        ClientId = settings.ClientId;
        ShowWindowOnStartup = settings.ShowWindowOnStartup;

        _baseline = CreateSnapshot();
        IsDirty = false;
    }

    public AppSettings ToSettings()
    {
        return new AppSettings
        {
            BrokerHost = BrokerHost,
            BrokerPort = BrokerPort,
            Username = Username,
            Password = Password,
            Namespace = Namespace,
            UseTls = UseTls,
            AutoStart = AutoStart,
            AutoCheckUpdates = AutoCheckUpdates,
            ClientId = ClientId,
            ShowWindowOnStartup = ShowWindowOnStartup
        };
    }

    public void MarkClean()
    {
        _baseline = CreateSnapshot();
        IsDirty = false;
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    private bool SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (Equals(field, value))
        {
            return false;
        }

        field = value;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        UpdateIsDirty();
        return true;
    }

    private void UpdateIsDirty()
    {
        var current = CreateSnapshot();
        IsDirty =
            !string.Equals(current.BrokerHost, _baseline.BrokerHost, StringComparison.Ordinal) ||
            current.BrokerPort != _baseline.BrokerPort ||
            !string.Equals(current.Username, _baseline.Username, StringComparison.Ordinal) ||
            !string.Equals(current.Password, _baseline.Password, StringComparison.Ordinal) ||
            current.UseTls != _baseline.UseTls ||
            current.AutoStart != _baseline.AutoStart ||
            !string.Equals(current.Namespace, _baseline.Namespace, StringComparison.Ordinal) ||
            current.AutoCheckUpdates != _baseline.AutoCheckUpdates ||
            !string.Equals(current.ClientId, _baseline.ClientId, StringComparison.Ordinal) ||
            current.ShowWindowOnStartup != _baseline.ShowWindowOnStartup;
    }

    private AppSettings CreateSnapshot()
    {
        return new AppSettings
        {
            BrokerHost = BrokerHost,
            BrokerPort = BrokerPort,
            Username = Username,
            Password = Password,
            UseTls = UseTls,
            AutoStart = AutoStart,
            Namespace = Namespace,
            AutoCheckUpdates = AutoCheckUpdates,
            ClientId = ClientId,
            ShowWindowOnStartup = ShowWindowOnStartup
        };
    }

    private static string GetAppVersion()
    {
        var version = Assembly.GetExecutingAssembly()
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
            ?? "unknown";

        var plusIndex = version.IndexOf('+');
        return plusIndex > 0 ? version[..plusIndex] : version;
    }
}
