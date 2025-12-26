using System.Text;

namespace HaWin.Utilities;

public static class TopicHelper
{
    public static string GetDeviceId()
    {
        var name = Environment.MachineName.Trim().ToLowerInvariant();
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
