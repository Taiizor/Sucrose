using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Clean
    {
        public static string FileName(string Name)
        {
            char[] Chars = Path.GetInvalidFileNameChars();

            return new(Name.Where(Char => !Chars.Contains(Char)).ToArray());
        }

        public static string PathName(string Name)
        {
            char[] Chars = Path.GetInvalidPathChars();

            return new(Name.Where(Char => !Chars.Contains(Char)).ToArray());
        }
    }
}