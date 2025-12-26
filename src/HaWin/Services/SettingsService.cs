using System.IO;
using System.Text.Json;
using HaWin.Models;

namespace HaWin.Services;

public class SettingsService
{
    private readonly string _settingsPath;
    private const string AppDataFolderName = "HA Win";
    private const string LegacyAppDataFolderName = "HaWin";

    public SettingsService()
    {
        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var folder = Path.Combine(appData, AppDataFolderName);
        var legacyFolder = Path.Combine(appData, LegacyAppDataFolderName);
        if (!Directory.Exists(folder) && Directory.Exists(legacyFolder))
        {
            Directory.CreateDirectory(folder);
            var legacySettingsPath = Path.Combine(legacyFolder, "settings.json");
            var newSettingsPath = Path.Combine(folder, "settings.json");
            if (File.Exists(legacySettingsPath) && !File.Exists(newSettingsPath))
            {
                File.Copy(legacySettingsPath, newSettingsPath);
            }
        }
        Directory.CreateDirectory(folder);
        _settingsPath = Path.Combine(folder, "settings.json");
    }

    public AppSettings Load()
    {
        if (!File.Exists(_settingsPath))
        {
            return new AppSettings();
        }

        var json = File.ReadAllText(_settingsPath);
        var settings = JsonSerializer.Deserialize<AppSettings>(json);
        return settings ?? new AppSettings();
    }

    public void Save(AppSettings settings)
    {
        var json = JsonSerializer.Serialize(settings, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(_settingsPath, json);
    }
}
