using System.Text;

namespace Sucrose.Mpv.NET.Player
{
    internal static class YouTubeDlHelperQuality
    {
        public static string GetFormatStringForVideoQuality(YouTubeDlVideoQuality videoQuality)
        {
            StringBuilder stringBuilder = new("bestvideo");

            if (videoQuality != YouTubeDlVideoQuality.Highest)
            {
                stringBuilder.Append("[height<=");
                stringBuilder.Append((int)videoQuality);
                stringBuilder.Append("]");
            }

            stringBuilder.Append("+bestaudio/best");

            return stringBuilder.ToString();
        }
    }
}