using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Delete
    {
        public static void Folder(string Source, string Exclude = null, bool Self = true)
        {
            if (Directory.Exists(Source))
            {
                foreach (string Record in Directory.GetFiles(Source))
                {
                    if (File.Exists(Record))
                    {
                        File.Delete(Record);
                    }
                }

                foreach (string SubDirectory in Directory.GetDirectories(Source))
                {
                    if (Directory.Exists(SubDirectory))
                    {
                        if (!string.Equals(Path.GetFileName(SubDirectory), Exclude, StringComparison.OrdinalIgnoreCase))
                        {
                            Folder(SubDirectory, Exclude, true);
                        }
                    }
                }

                if (!string.Equals(Path.GetFileName(Source), Exclude, StringComparison.OrdinalIgnoreCase))
                {
                    if (Self)
                    {
                        Directory.Delete(Source, true);
                    }
                }
            }
        }
    }
}