using Microsoft.Win32;

namespace HaWin.Services;

public class AutoStartService
{
    private const string RunKeyPath = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    private const string AppName = "HaWin";

    public void SetAutoStart(bool enabled, string exePath)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true) ??
                        Registry.CurrentUser.CreateSubKey(RunKeyPath, true);

        if (enabled)
        {
            key?.SetValue(AppName, '"' + exePath + '"');
        }
        else
        {
            key?.DeleteValue(AppName, false);
        }
    }

    public bool IsAutoStartEnabled()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, false);
        if (key == null)
        {
            return false;
        }

        return key.GetValue(AppName) != null;
    }
}
