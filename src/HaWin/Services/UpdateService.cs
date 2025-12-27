using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace HaWin.Services;

public sealed class UpdateService
{
    private const string ReleasesUrl = "https://api.github.com/repos/derDeno/HA-Win/releases/latest";
    private static readonly HttpClient Http = CreateClient();

    public async Task<UpdateInfo?> CheckForUpdateAsync(string currentVersion, CancellationToken cancellationToken = default)
    {
        if (!TryParseVersion(currentVersion, out var current))
        {
            return null;
        }

        using var response = await Http.GetAsync(ReleasesUrl, cancellationToken);
        response.EnsureSuccessStatusCode();

        await using var stream = await response.Content.ReadAsStreamAsync(cancellationToken);
        var release = await JsonSerializer.DeserializeAsync<ReleaseResponse>(stream, cancellationToken: cancellationToken);
        if (release == null || string.IsNullOrWhiteSpace(release.TagName))
        {
            return null;
        }

        if (!TryParseVersion(release.TagName, out var latest) || latest <= current)
        {
            return null;
        }

        var asset = release.Assets?
            .FirstOrDefault(a => a.Name != null
                                 && a.Name.Contains("setup", StringComparison.OrdinalIgnoreCase)
                                 && a.Name.EndsWith(".exe", StringComparison.OrdinalIgnoreCase))
            ?? release.Assets?.FirstOrDefault(a => a.Name != null && a.Name.EndsWith(".exe", StringComparison.OrdinalIgnoreCase))
            ?? release.Assets?.FirstOrDefault(a => a.BrowserDownloadUrl != null && a.BrowserDownloadUrl.EndsWith(".exe", StringComparison.OrdinalIgnoreCase));

        if (asset?.BrowserDownloadUrl == null)
        {
            return null;
        }

        return new UpdateInfo(latest.ToString(), asset.BrowserDownloadUrl);
    }

    public async Task DownloadAndRunInstallerAsync(string downloadUrl, CancellationToken cancellationToken = default)
    {
        var fileName = Path.GetFileName(new Uri(downloadUrl).LocalPath);
        if (string.IsNullOrWhiteSpace(fileName))
        {
            fileName = "HA-Win-Setup.exe";
        }

        var downloadPath = Path.Combine(Path.GetTempPath(), fileName);
        await using var stream = await Http.GetStreamAsync(downloadUrl, cancellationToken);
        await using var file = File.Create(downloadPath);
        await stream.CopyToAsync(file, cancellationToken);

        Process.Start(new ProcessStartInfo(downloadPath) { UseShellExecute = true });
    }

    public string GetCurrentVersion()
    {
        var version = Assembly.GetExecutingAssembly()
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
            ?? "0.0.0";

        var plusIndex = version.IndexOf('+');
        return plusIndex > 0 ? version[..plusIndex] : version;
    }

    private static HttpClient CreateClient()
    {
        var client = new HttpClient();
        client.DefaultRequestHeaders.UserAgent.ParseAdd("HA-Win-Updater");
        return client;
    }

    private static bool TryParseVersion(string value, out Version version)
    {
        var trimmed = value.Trim();
        if (trimmed.StartsWith("v", StringComparison.OrdinalIgnoreCase))
        {
            trimmed = trimmed[1..];
        }

        var end = trimmed.IndexOfAny(['+', '-']);
        if (end > 0)
        {
            trimmed = trimmed[..end];
        }

        return Version.TryParse(trimmed, out version);
    }

    private sealed class ReleaseResponse
    {
        [JsonPropertyName("tag_name")]
        public string? TagName { get; set; }

        [JsonPropertyName("assets")]
        public ReleaseAsset[]? Assets { get; set; } = Array.Empty<ReleaseAsset>();
    }

    private sealed class ReleaseAsset
    {
        [JsonPropertyName("name")]
        public string? Name { get; set; }

        [JsonPropertyName("browser_download_url")]
        public string? BrowserDownloadUrl { get; set; }
    }

    public sealed record UpdateInfo(string LatestVersion, string DownloadUrl);
}
