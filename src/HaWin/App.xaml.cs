using System.Windows;
using HaWin.Models;
using HaWin.Services;

namespace HaWin;

public partial class App : System.Windows.Application
{
    public SettingsService SettingsService { get; } = new();
    public AutoStartService AutoStartService { get; } = new();
    public TrayIconService TrayIconService { get; private set; } = null!;
    public NotificationService NotificationService { get; private set; } = null!;
    public MqttService MqttService { get; private set; } = null!;
    public UpdateService UpdateService { get; } = new();

    private AppSettings _settings = new();

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        TrayIconService = new TrayIconService(ShowSettingsWindow, ExitApplication);
        NotificationService = new NotificationService(TrayIconService);
        NotificationService.Initialize();

        _settings = SettingsService.Load();
        _settings.AutoStart = AutoStartService.IsAutoStartEnabled();
        SettingsService.Save(_settings);

        MqttService = new MqttService(NotificationService);
        _ = MqttService.ApplySettingsAsync(_settings);

        var window = new MainWindow();
        Current.MainWindow = window;
        window.Show();
    }

    private void ShowSettingsWindow()
    {
        if (Current.MainWindow == null)
        {
            Current.MainWindow = new MainWindow();
        }

        Current.MainWindow.Show();
        Current.MainWindow.WindowState = WindowState.Normal;
        Current.MainWindow.Activate();
    }

    private void ExitApplication()
    {
        if (Current.MainWindow is MainWindow window)
        {
            window.AllowClose();
        }

        Shutdown();
    }

    public void RequestExit()
    {
        ExitApplication();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        TrayIconService.Dispose();
        MqttService.Dispose();
        base.OnExit(e);
    }

}
