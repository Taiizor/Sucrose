using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHU = Sucrose.Shared.Space.Helper.User;

namespace Sucrose.Portal.Extension
{
    internal static class ThemeCreate
    {
        public static string GetAuthor()
        {
            return SSSHU.GetName();
        }

        public static string GetContact()
        {
            string Result;

            if (SMMRG.Randomise.Next(2) == 0)
            {
                Result = $"{GetAuthor()}@example.com";
            }
            else
            {
                Result = $"https://{GetAuthor()}.example.com";
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