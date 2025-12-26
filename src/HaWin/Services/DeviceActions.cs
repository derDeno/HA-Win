using System.Diagnostics;
using System.Windows.Forms;

namespace HaWin.Services;

public static class DeviceActions
{
    public static void Restart()
    {
        Process.Start(new ProcessStartInfo("shutdown", "/r /t 0") { CreateNoWindow = true, UseShellExecute = false });
    }

    public static void Shutdown()
    {
        Process.Start(new ProcessStartInfo("shutdown", "/s /t 0") { CreateNoWindow = true, UseShellExecute = false });
    }

    public static void Standby()
    {
        Application.SetSuspendState(PowerState.Suspend, true, true);
    }
}
