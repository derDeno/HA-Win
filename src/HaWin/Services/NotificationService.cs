using CommunityToolkit.WinUI.Notifications;
using HaWin.Utilities;

namespace HaWin.Services;

public class NotificationService
{
    private const string AppId = "HA Win";
    private readonly TrayIconService _trayIcon;

    public NotificationService(TrayIconService trayIcon)
    {
        _trayIcon = trayIcon;
    }

    public void Initialize()
    {
        ShortcutHelper.EnsureShortcut(AppId);
    }

    public void ShowNotification(string title, string message)
    {
        try
        {
            new ToastContentBuilder()
                .AddText(title)
                .AddText(message)
                .Show();
        }
        catch
        {
            _trayIcon.ShowBalloon(title, message);
        }
    }
}
