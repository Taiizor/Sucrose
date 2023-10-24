namespace Sucrose.Shared.Space.Helper
{
    internal static class Tags
    {
        public static string Join(string[] Tags, string Separator, string Back)
        {
            if (Tags == null || !Tags.Any())
            {
                return Back;
            }
            else
            {
                return string.Join(Separator, Tags.Where(Tag => !string.IsNullOrEmpty(Tag))).ToLowerInvariant();
            }
        }
    }
}