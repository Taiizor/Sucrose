using System.Text.RegularExpressions;

namespace Sucrose.Theme.Shared.Helper
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
    }
}