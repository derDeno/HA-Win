using System.Diagnostics;
using System.Windows;
using HaWin.Models;
using HaWin.ViewModels;

namespace HaWin;

public partial class MainWindow : Window
{
    private readonly SettingsViewModel _viewModel = new();
    private readonly App _app;
    private bool _allowClose;

    public MainWindow()
    {
        InitializeComponent();
        _app = (App)System.Windows.Application.Current;

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

    private void OnConnectionChanged(object? sender, bool isConnected)
    {
        Dispatcher.Invoke(() => _viewModel.IsConnected = isConnected);
    }

    private async Task UpdateConnectionStatusAsync()
    {
        await Task.Delay(800);
        _viewModel.IsConnected = _app.MqttService.IsConnected;
    }

    public void AllowClose()
    {
        _allowClose = true;
        Close();
    }
}
