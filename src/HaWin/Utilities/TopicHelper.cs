using System.Text;

namespace HaWin.Utilities;

public static class TopicHelper
{
    public static string GetDeviceId()
    {
        return SanitizeNamespace(Environment.MachineName);
    }

    public static string SanitizeNamespace(string value)
    {
        var name = value.Trim().ToLowerInvariant();
        var sb = new StringBuilder(name.Length);

        foreach (var ch in name)
        {
            if (char.IsLetterOrDigit(ch))
            {
                sb.Append(ch);
            }
            else if (ch == '-' || ch == '_' || ch == ' ')
            {
                sb.Append('_');
            }
        }

        var result = sb.ToString();
        return string.IsNullOrWhiteSpace(result) ? "pc" : result;
    }
}
