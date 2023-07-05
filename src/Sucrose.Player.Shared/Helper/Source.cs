using System.IO;
using System.Text.RegularExpressions;
#if NET48_OR_GREATER
using System.Net.Http;
using SMR = Sucrose.Memory.Readonly;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
#endif

namespace Sucrose.Player.Shared.Helper
{
    internal static class Source
    {
        public static Uri GetSource(Uri Source)
        {
            return GetSource(Source.ToString());
        }

        public static Uri GetSource(string Source)
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
                    return new Uri(@LocalSource, UriKind.RelativeOrAbsolute);
                }
                else
                {
                    using HttpClient Client = new();
                    using HttpResponseMessage Response = Client.GetAsync(Source).Result;
                    using Stream Content = Response.Content.ReadAsStreamAsync().Result;
                    using FileStream Stream = new(LocalSource, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                    Content.CopyTo(Stream);

                    return new Uri(@Path.GetFullPath(LocalSource), UriKind.RelativeOrAbsolute);
                }
#else
                return new Uri(@Source, UriKind.RelativeOrAbsolute);
#endif
            }
            else
            {
                return new Uri(@Source, UriKind.RelativeOrAbsolute);
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