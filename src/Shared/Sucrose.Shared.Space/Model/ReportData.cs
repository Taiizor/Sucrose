namespace Sucrose.Shared.Space.Model
{
    internal class ReportData(string AppVersion, string ContactEmail, string WallpaperTitle, string RelatedCategory, string WallpaperVersion, string WallpaperLocation, string DescriptionMessage, string WallpaperAppVersion)
    {
        public string AppVersion { get; set; } = AppVersion;

        public string ContactEmail { get; set; } = ContactEmail;

        public string WallpaperTitle { get; set; } = WallpaperTitle;

        public string RelatedCategory { get; set; } = RelatedCategory;

        public string WallpaperVersion { get; set; } = WallpaperVersion;

        public string WallpaperLocation { get; set; } = WallpaperLocation;

        public string DescriptionMessage { get; set; } = DescriptionMessage;

        public string WallpaperAppVersion { get; set; } = WallpaperAppVersion;
    }
}