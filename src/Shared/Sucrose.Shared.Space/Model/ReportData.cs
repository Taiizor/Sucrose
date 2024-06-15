namespace Sucrose.Shared.Space.Model
{
    public class ReportData(string Title, string Contact, string Version, string Category, string Location, string AppVersion, string Description, string ThemeVersion)
    {
        public string Title { get; set; } = Title;

        public string Contact { get; set; } = Contact;

        public string Version { get; set; } = Version;

        public string Category { get; set; } = Category;

        public string Location { get; set; } = Location;

        public string AppVersion { get; set; } = AppVersion;

        public string Description { get; set; } = Description;

        public string ThemeVersion { get; set; } = ThemeVersion;
    }
}