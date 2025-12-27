namespace HaWin.Models;

public class AppSettings
{
    public string BrokerHost { get; set; } = "";
    public int BrokerPort { get; set; } = 1883;
    public string Username { get; set; } = "";
    public string Password { get; set; } = "";
    public bool UseTls { get; set; }
    public bool AutoStart { get; set; }
    public bool AutoCheckUpdates { get; set; }
    public string ClientId { get; set; } = "";
}
