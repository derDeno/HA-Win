using Microsoft.Win32;

namespace HaWin.Services;

public class AutoStartService
{
    private const string RunKeyPath = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    private const string AppName = "HA Win";
    private const string LegacyAppName = "HaWin";

    public void SetAutoStart(bool enabled, string exePath)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true) ??
                        Registry.CurrentUser.CreateSubKey(RunKeyPath, true);

        if (enabled)
        {
            key?.SetValue(AppName, '"' + exePath + '"');
            if (AppName != LegacyAppName)
            {
                key?.DeleteValue(LegacyAppName, false);
            }
        }
        else
        {
            key?.DeleteValue(AppName, false);
            if (AppName != LegacyAppName)
            {
                key?.DeleteValue(LegacyAppName, false);
            }
        }
    }

    public bool IsAutoStartEnabled()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, false);
        if (key == null)
        {
            return false;
        }

        return key.GetValue(AppName) != null || key.GetValue(LegacyAppName) != null;
    }
}
