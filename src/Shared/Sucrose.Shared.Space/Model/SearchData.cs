namespace Sucrose.Shared.Space.Model
{
    internal class SearchData(string ActivePage, string AppVersion, string SearchQuery)
    {

        public string ActivePage { get; set; } = ActivePage;

        public string AppVersion { get; set; } = AppVersion;

        public string SearchQuery { get; set; } = SearchQuery;
    }
}