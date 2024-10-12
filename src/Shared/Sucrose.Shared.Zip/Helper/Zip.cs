using System.IO;
using System.IO.Compression;

namespace Sucrose.Shared.Zip.Helper
{
    internal static class Zip
    {
        public static bool CheckFolder(string Archive)
        {
            try
            {
                using ZipArchive Archives = ZipFile.OpenRead(Archive);

                if (Archives.Entries.Count > 0)
                {
                    ZipArchiveEntry Entry = Archives.Entries.First();

                    if (Entry.FullName.EndsWith(@"/"))
                    {
                        return false;
                    }
                }

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static bool CheckArchive(string Archive)
        {
            try
            {
                using FileStream Stream = new(Archive, FileMode.Open, FileAccess.Read, FileShare.Read);

                byte[] ArchiveHeader = new byte[4];

                _ = Stream.Read(ArchiveHeader, 0, 4);

                byte[] ZipHeader = new byte[] { 0x50, 0x4B, 0x03, 0x04 };

                return ArchiveHeader.SequenceEqual(ZipHeader);
            }
            catch
            {
                return false;
            }
        }

        public static bool EncryptedArchive(string Archive)
        {
            try
            {
                using FileStream Stream = new(Archive, FileMode.Open, FileAccess.Read, FileShare.Read);

                byte[] ArchiveHeader = new byte[4];

                _ = Stream.Read(ArchiveHeader, 0, 4);

                byte[] ZipHeader = new byte[] { 0x50, 0x4B, 0x03, 0x07 };

                return ArchiveHeader.SequenceEqual(ZipHeader);
            }
            catch
            {
                return false;
            }
        }

        public static string EntryName(string File, string Source)
        {
            try
            {
#if NET48_OR_GREATER
                string Relative = File.Substring(Source.Length);
#else

                string Relative = File[Source.Length..];
#endif

                Relative = Relative.TrimStart(Path.DirectorySeparatorChar);

                return Path.Combine(Path.GetFileName(Path.GetDirectoryName(File)), Relative);
            }
            catch
            {
                return string.Empty;
            }
        }

        public static string ReadFile(string Archive, string File)
        {
            try
            {
                using ZipArchive Archives = ZipFile.OpenRead(Archive);

                ZipArchiveEntry Entry = Archives.GetEntry(File);

                using StreamReader Reader = new(Entry.Open());

                return Reader.ReadToEnd();
            }
            catch
            {
                return string.Empty;
            }
        }

        public static bool CheckFile(string Archive, string File, StringComparison Comparison = StringComparison.Ordinal)
        {
            try
            {
                using ZipArchive Archives = ZipFile.OpenRead(Archive);

                return Archives.Entries.Any(Entry => string.Equals(Entry.Name, File, Comparison) || string.Equals(Entry.FullName, File, Comparison));
            }
            catch
            {
                return false;
            }
        }
    }
}