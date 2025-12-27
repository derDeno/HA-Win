using System.Diagnostics;
using System.Windows;
using HaWin.Models;
using HaWin.Services;
using HaWin.ViewModels;

namespace HaWin;

public partial class MainWindow : Window
{
    private readonly SettingsViewModel _viewModel = new();
    private readonly App _app;
    private readonly UpdateService _updateService;
    private bool _allowClose;

    public MainWindow()
    {
        InitializeComponent();
        _app = (App)System.Windows.Application.Current;
        _updateService = _app.UpdateService;

        DataContext = _viewModel;

        var settings = _app.SettingsService.Load();
        _viewModel.LoadFrom(settings);
        PasswordBox.Password = settings.Password;

        _app.MqttService.ConnectionChanged += OnConnectionChanged;
        _viewModel.IsConnected = _app.MqttService.IsConnected;
        _ = UpdateConnectionStatusAsync();
    }

    protected override void OnClosing(System.ComponentModel.CancelEventArgs e)
    {
        if (!_allowClose)
        {
            e.Cancel = true;
            Hide();
            return;
        }

        base.OnClosing(e);
    }

    private async void SaveButton_OnClick(object sender, RoutedEventArgs e)
    {
        var settings = _viewModel.ToSettings();
        settings.Password = PasswordBox.Password;

        _app.SettingsService.Save(settings);

        var exePath = Environment.ProcessPath ?? Process.GetCurrentProcess().MainModule?.FileName;
        if (!string.IsNullOrWhiteSpace(exePath))
        {
            _app.AutoStartService.SetAutoStart(settings.AutoStart, exePath);
        }

        await _app.MqttService.ApplySettingsAsync(settings);
    }

    private async void TestConnectionButton_OnClick(object sender, RoutedEventArgs e)
    {
        var settings = _viewModel.ToSettings();
        settings.Password = PasswordBox.Password;

        var success = await _app.MqttService.TestConnectionAsync(settings);
        var caption = "MQTT Test";
        var message = success ? "Connection successful." : "Connection failed.";
        var icon = success ? MessageBoxImage.Information : MessageBoxImage.Warning;

        System.Windows.MessageBox.Show(this, message, caption, MessageBoxButton.OK, icon);
    }

    private void HideButton_OnClick(object sender, RoutedEventArgs e)
    {
        Hide();
    }

    private void OpenGitHubButton_OnClick(object sender, RoutedEventArgs e)
    {
        var url = "https://github.com/derDeno/HA-Win";
        Process.Start(new ProcessStartInfo(url) { UseShellExecute = true });
    }

    private async void CheckUpdatesButton_OnClick(object sender, RoutedEventArgs e)
    {
        await CheckForUpdatesAsync();
    }

    private void OnConnectionChanged(object? sender, bool isConnected)
    {
        Dispatcher.Invoke(() => _viewModel.IsConnected = isConnected);
    }

    private async Task UpdateConnectionStatusAsync()
    {
        await Task.Delay(800);
        _viewModel.IsConnected = _app.MqttService.IsConnected;
    }

    private async Task CheckForUpdatesAsync()
    {
        try
        {
            var result = await _updateService.CheckForUpdateAsync(_updateService.GetCurrentVersion());
            if (result == null)
            {
                System.Windows.MessageBox.Show(this, "You are running the latest version.", "Update Check",
                    MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            var message = $"A new version ({result.LatestVersion}) is available. Download and install it now?";
            var choice = System.Windows.MessageBox.Show(this, message, "Update Available",
                MessageBoxButton.YesNo, MessageBoxImage.Information);
            if (choice == MessageBoxResult.Yes)
            {
                await _updateService.DownloadAndRunInstallerAsync(result.DownloadUrl);
            }
        }
        catch (Exception ex)
        {
            System.Windows.MessageBox.Show(this, $"Update check failed: {ex.Message}", "Update Check",
                MessageBoxButton.OK, MessageBoxImage.Error);
        }
    }

    public void AllowClose()
    {
        _allowClose = true;
        Close();
    }
}
