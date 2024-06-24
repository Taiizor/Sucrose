namespace Sucrose.Shared.Space.Model
{
    internal class StoreData(string Title, string Version, string Location, string AppVersion, string ThemeVersion)
    {
        public string Title { get; set; } = Title;

        public string Version { get; set; } = Version;

        public string Location { get; set; } = Location;

        public string AppVersion { get; set; } = AppVersion;

        public string ThemeVersion { get; set; } = ThemeVersion;
    }
}