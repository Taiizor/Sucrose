using System.Collections.Specialized;
using System.Text.RegularExpressions;
using System.Web;

namespace Sucrose.Shared.Theme.Helper
{
    internal static class Various
    {
        public static bool IsUrl(string Address)
        {
            string Pattern = @"^(http|https)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?$";
            //string Pattern = @"^(http|https|ftp)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?$";

            Regex Regex = new(Pattern, RegexOptions.IgnoreCase);

            return Regex.IsMatch(Address);
        }

        public static bool IsMail(string Mail)
        {
            string Pattern = @"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

            return Regex.IsMatch(Mail, Pattern);
        }

        public static bool IsYouTubeVideo(string Address)
        {
            return IsYouTubeUrl(Address) || IsYouTubeMusicUrl(Address);
        }

        public static bool IsYouTubePlaylist(string Address)
        {
            return IsYouTubePlaylistUrl(Address) || IsYouTubeMusicPlaylistUrl(Address);
        }

        public static bool IsYouTube(string Address)
        {
            return IsYouTubeUrl(Address) || IsYouTubePlaylistUrl(Address);
        }

        public static bool IsYouTubeMusic(string Address)
        {
            return IsYouTubeMusicUrl(Address) || IsYouTubeMusicPlaylistUrl(Address);
        }

        public static bool IsYouTubeUrl(string Address)
        {
            if (IsUrl(Address))
            {
                string Pattern = @"^(https?://)?(www\.)?(youtube\.com/watch\?v=)";
                return Regex.IsMatch(Address, Pattern);
            }
            else
            {
                return false;
            }
        }

        public static bool IsYouTubeMusicUrl(string Address)
        {
            if (IsUrl(Address))
            {
                string Pattern = @"^(https?://)?(music\.)?(youtube\.com/watch\?v=)";
                return Regex.IsMatch(Address, Pattern);
            }
            else
            {
                return false;
            }
        }

        public static bool IsYouTubePlaylistUrl(string Address)
        {
            if (IsUrl(Address))
            {
                string Pattern = @"^(https?://)?(www\.)?(youtube\.com/playlist\?list=)";
                return Regex.IsMatch(Address, Pattern);
            }
            else
            {
                return false;
            }
        }

        public static bool IsYouTubeMusicPlaylistUrl(string Address)
        {
            if (IsUrl(Address))
            {
                string Pattern = @"^(https?://)?(music\.)?(youtube\.com/playlist\?list=)";
                return Regex.IsMatch(Address, Pattern);
            }
            else
            {
                return false;
            }
        }

        public static string GetYouTubeVideoId(string Address)
        {
            if (Uri.TryCreate(Address, UriKind.Absolute, out Uri Url))
            {
                NameValueCollection QueryParameters = HttpUtility.ParseQueryString(Url.Query);
                return QueryParameters["v"];
            }

            return string.Empty;
        }

        public static string GetYouTubePlaylistId(string Address)
        {
            if (Uri.TryCreate(Address, UriKind.Absolute, out Uri Url))
            {
                NameValueCollection QueryParameters = HttpUtility.ParseQueryString(Url.Query);
                return QueryParameters["list"];
            }

            return string.Empty;
        }
    }
}