namespace Sucrose.Shared.Space.Model
{
    internal class SearchData(string Page, string Query, string Version)
    {
        public string Page { get; set; } = Page;

        public string Query { get; set; } = Query;

        public string Version { get; set; } = Version;
    }
}