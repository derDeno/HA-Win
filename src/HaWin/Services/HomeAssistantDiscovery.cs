using System.Reflection;
using System.Text.Json;

namespace HaWin.Services;

public class HomeAssistantDiscovery
{
    public string DiscoveryPrefix { get; } = "homeassistant";

    public string BaseTopic(string deviceId) => $"ha-win/{deviceId}";

    public string StatusTopic(string deviceId) => $"{BaseTopic(deviceId)}/status";

    public string NotifyCommandTopic(string deviceId) => $"{BaseTopic(deviceId)}/notify";

    public string ButtonCommandTopic(string deviceId, string action) => $"{BaseTopic(deviceId)}/{action}/set";

    public string ButtonConfigTopic(string deviceId, string action) => $"{DiscoveryPrefix}/button/{deviceId}/{action}/config";

    public string NotifyConfigTopic(string deviceId) => $"{DiscoveryPrefix}/notify/{deviceId}/config";

    public string BuildButtonConfig(string deviceId, string machineName, string action, string name, string icon)
    {
        var payload = new
        {
            name,
            unique_id = $"{deviceId}_{action}",
            command_topic = ButtonCommandTopic(deviceId, action),
            payload_press = "PRESS",
            availability_topic = StatusTopic(deviceId),
            payload_available = "online",
            payload_not_available = "offline",
            icon,
            device = BuildDeviceInfo(deviceId, machineName)
        };

        return JsonSerializer.Serialize(payload);
    }

    public string BuildNotifyConfig(string deviceId, string machineName)
    {
        var payload = new
        {
            name = "Notification",
            unique_id = $"{deviceId}_notify",
            command_topic = NotifyCommandTopic(deviceId),
            availability_topic = StatusTopic(deviceId),
            payload_available = "online",
            payload_not_available = "offline",
            device = BuildDeviceInfo(deviceId, machineName)
        };

        return JsonSerializer.Serialize(payload);
    }

    private static object BuildDeviceInfo(string deviceId, string machineName)
    {
        var version = Assembly.GetExecutingAssembly()
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
            ?? Assembly.GetExecutingAssembly().GetName().Version?.ToString()
            ?? "unknown";

        return new
        {
            identifiers = new[] { deviceId },
            name = machineName,
            manufacturer = "DNO",
            model = "HA-Win",
            sw_version = version
        };
    }
}
