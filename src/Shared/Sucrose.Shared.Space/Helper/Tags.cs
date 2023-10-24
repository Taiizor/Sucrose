using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Tags
    {
        public static string Join(string[] Tags)
        {
            if (Tags == null || !Tags.Any())
            {
                return string.Empty;
            }
            else
            {
                return string.Join($"{SMR.SearchSplit}", Tags.Where(Tag => !string.IsNullOrEmpty(Tag))).ToLowerInvariant();
            }
        }
    }
}