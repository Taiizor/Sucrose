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
    }
}