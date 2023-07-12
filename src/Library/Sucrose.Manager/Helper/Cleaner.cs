namespace Sucrose.Manager.Helper
{
    internal static class Cleaner
    {
        public static string Clean(string Content)
        {
            try
            {
                if (Content.StartsWith("{{"))
                {
#if NET48_OR_GREATER || NETSTANDARD2_0
                    Content = Content.Substring(1);
#else
                    Content = Content[1..];
#endif
                }

                if (Content.EndsWith("}}"))
                {
#if NET48_OR_GREATER || NETSTANDARD2_0
                    Content = Content.Substring(0, Content.Length - 1);
#else
                    Content = Content[..^1];
#endif
                }

                return Content;
            }
            catch
            {
                return string.Empty;
            }
        }
    }
}