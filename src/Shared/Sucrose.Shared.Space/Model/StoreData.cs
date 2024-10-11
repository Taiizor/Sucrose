namespace Sucrose.Shared.Space.Model
{
    internal class StoreData(string AppVersion, string WallpaperTitle, string WallpaperVersion, string WallpaperLocation, string WallpaperAppVersion)
    {
        public string AppVersion { get; set; } = AppVersion;

        public string WallpaperTitle { get; set; } = WallpaperTitle;

        public string WallpaperVersion { get; set; } = WallpaperVersion;

        public string WallpaperLocation { get; set; } = WallpaperLocation;

        public string WallpaperAppVersion { get; set; } = WallpaperAppVersion;
    }
}