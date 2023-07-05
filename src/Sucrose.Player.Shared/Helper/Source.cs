using System.IO;
using System.Text.RegularExpressions;
using SMR = Sucrose.Memory.Readonly;
#if NET48_OR_GREATER
using System.Net.Http;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
#endif

namespace Sucrose.Player.Shared.Helper
{
    internal static class Source
    {
        public static bool GetExtension(Uri Source)
        {
            return GetExtension(Source.ToString());
        }

        public static bool GetExtension(string Source)
        {
            if (Path.GetExtension(Source) != ".mov")
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        public static string GetContent(Uri Source)
        {
            return GetContent(Source.ToString());
        }

        public static string GetContent(string Source)
        {
            return $"<html><head><meta name=\"viewport\" content=\"width=device-width\"></head><body><video autoplay name=\"media\" src=\"{Source}\"></video></body></html>";
        }

        public static void WriteContent(string ContentPath, Uri Content)
        {
            WriteContent(ContentPath, Content.ToString());
        }

        public static void WriteContent(string ContentPath, string Content)
        {
            File.WriteAllText(ContentPath, GetContent(Content));
        }

        public static string GetContentPath()
        {
            return Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.WebView2, SMR.VideoContent);
        }

        public static Uri GetSource(Uri Source)
        {
            return GetSource(Source.ToString());
        }

        public static Uri GetSource(string Source, UriKind Kind = UriKind.RelativeOrAbsolute)
        {
            if (IsUrl(Source))
            {
#if NET48_OR_GREATER
                string CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.MediaElement);

                if (!Directory.Exists(CachePath))
                {
                    Directory.CreateDirectory(CachePath);
                }

                //string LocalSource = @Path.Combine(CachePath, Path.ChangeExtension(Path.GetRandomFileName(), Path.GetExtension(Source)));
                string LocalSource = @Path.Combine(CachePath, $"{SSECCE.TextToMD5(Source)}{Path.GetExtension(Source)}");

                if (File.Exists(LocalSource))
                {
                    return new Uri(@LocalSource, Kind);
                }
                else
                {
                    using HttpClient Client = new();
                    using HttpResponseMessage Response = Client.GetAsync(Source).Result;
                    using Stream Content = Response.Content.ReadAsStreamAsync().Result;
                    using FileStream Stream = new(LocalSource, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                    Content.CopyTo(Stream);

                    return new Uri(@Path.GetFullPath(LocalSource), Kind);
                }
#else
                return new Uri(@Source, Kind);
#endif
            }
            else
            {
                return new Uri(@Source, Kind);
            }
        }

        public static bool IsUrl(string Address)
        {
            string Pattern = @"^(http|https|ftp)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?$";

            Regex Regex = new(Pattern, RegexOptions.IgnoreCase);

            return Regex.IsMatch(Address);
        }
    }
}