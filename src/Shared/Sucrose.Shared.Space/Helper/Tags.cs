namespace Sucrose.Shared.Space.Helper
{
    internal static class Tags
    {
        public static string Join(string[] Tags, string Separator, bool Lower, string Back)
        {
            if (Tags == null || !Tags.Any())
            {
                return Back;
            }
            else
            {
                if (Lower)
                {
                    return string.Join(Separator, Tags.Where(Tag => !string.IsNullOrEmpty(Tag))).ToLowerInvariant();
                }
                else
                {
                    return string.Join(Separator, Tags.Where(Tag => !string.IsNullOrEmpty(Tag)));
                }
            }
        }
    }
}