using System.IO;
using SMR = Sucrose.Memory.Readonly;
using STSHV = Sucrose.Theme.Shared.Helper.Various;
#if NET48_OR_GREATER
using System.Net.Http;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
#endif

namespace Sucrose.Engine.Shared.Helper
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

        public static string GetVideoContent(Uri Source)
        {
            return GetVideoContent(Source.ToString());
        }

        public static string GetVideoContent(string Source)
        {
            return $"<html><head><meta name=\"viewport\" content=\"width=device-width\"></head><body><video autoplay name=\"media\" src=\"{Source}\"></video></body></html>";
        }

        public static void WriteVideoContent(string VideoContentPath, Uri Content)
        {
            WriteVideoContent(VideoContentPath, Content.ToString());
        }

        public static void WriteVideoContent(string VideoContentPath, string Content)
        {
            File.WriteAllText(VideoContentPath, GetVideoContent(Content));
        }

        public static string GetVideoContentPath()
        {
            return Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content, SMR.VideoContent);
        }

        public static Uri GetSource(Uri Source)
        {
            return GetSource(Source.ToString());
        }

        public static Uri GetSource(string Source, UriKind Kind = UriKind.RelativeOrAbsolute)
        {
            if (STSHV.IsUrl(Source))
            {
#if NET48_OR_GREATER
                string CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content);

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
    }
}