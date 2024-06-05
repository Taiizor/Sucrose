namespace Sucrose.Shared.Space.Model
{
    public class ReportData(string Title, string Category, string Location, string Description)
    {
        public string Title { get; set; } = Title;

        public string Category { get; set; } = Category;

        public string Location { get; set; } = Location;

        public string Description { get; set; } = Description;
    }
}