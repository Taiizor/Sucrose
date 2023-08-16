using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Size
    {
        public static long Calc(string Path, SearchOption Option = SearchOption.AllDirectories)
        {
            try
            {
                if (Directory.Exists(Path))
                {
                    string[] CacheFiles = Directory.GetFiles(Path, "*.*", Option);

                    return CacheFiles.Sum(CacheFile => new FileInfo(CacheFile).Length);
                }
                else if (File.Exists(Path))
                {
                    return new FileInfo(Path).Length;
                }
                else
                {
                    return 0;
                }
            }
            catch
            {
                return 0;
            }
        }
    }
}