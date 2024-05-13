using SMR = Sucrose.Memory.Readonly;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;

namespace Sucrose.Portal.Extension
{
    internal static class ThemeCreate
    {
        public static string GetAuthor()
        {
            return Environment.UserName;
        }

        public static string GetContact()
        {
            string Result = string.Empty;

            if (SMR.Randomise.Next(2) == 0)
            {
                Result = $"{GetAuthor()}@example.com";
            }
            else
            {
                Result = $"https://{GetAuthor()}.com";
            }

            return Result.ToLowerInvariant();
        }

        public static string GetTitle(string Name)
        {
            return Name;
        }

        public static string GetDescription(string Name, SSDEWT Type)
        {
            return $"{Name} - {Type} Wallpaper";
        }
    }
}