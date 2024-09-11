using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Extension
    {
        public static string Change(string Source, string Extension)
        {
            return Path.ChangeExtension(Source, Extension);
        }
    }
}