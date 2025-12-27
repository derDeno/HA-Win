using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace HaWin.Services;

public class TrayIconService : IDisposable
{
    private readonly NotifyIcon _notifyIcon;
    private readonly Action _openAction;
    private readonly Action _exitAction;

    public TrayIconService(Action openAction, Action exitAction)
    {
        _openAction = openAction;
        _exitAction = exitAction;

        _notifyIcon = new NotifyIcon
        {
            Icon = LoadTrayIcon(),
            Text = "HA Win",
            Visible = true
        };

        var menu = new ContextMenuStrip();
        menu.Items.Add("Open", null, (_, _) => _openAction());
        menu.Items.Add("Exit", null, (_, _) => _exitAction());

        _notifyIcon.ContextMenuStrip = menu;
        _notifyIcon.DoubleClick += (_, _) => _openAction();
    }

    public void ShowBalloon(string title, string message)
    {
        _notifyIcon.ShowBalloonTip(5000, title, message, ToolTipIcon.Info);
    }

    private static Icon LoadTrayIcon()
    {
        try
        {
            var iconPath = Path.Combine(AppContext.BaseDirectory, "icon.ico");
            if (File.Exists(iconPath))
            {
                return new Icon(iconPath);
            }
        }
        catch
        {
            // Fall back to the system icon if loading fails.
        }

        return SystemIcons.Application;
    }

    public void Dispose()
    {
        _notifyIcon.Visible = false;
        _notifyIcon.Dispose();
    }
}
