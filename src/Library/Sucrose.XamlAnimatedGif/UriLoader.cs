using System.IO;
using System.IO.Packaging;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text;
using System.Windows;
using Sucrose.XamlAnimatedGif.Extensions;

namespace Sucrose.XamlAnimatedGif
{
    internal class UriLoader
    {
        public static TimeSpan DownloadCacheExpiration { get; set; } = TimeSpan.FromDays(30);
        public static string DownloadCacheLocation { get; set; } = Path.GetTempPath();
        public static string ClientUserAgent { get; set; } = "XamlAnimatedGif";

        public static Task<Stream> GetStreamFromUriAsync(Uri uri, IProgress<int> progress)
        {
            if (uri.IsAbsoluteUri && (uri.Scheme == "http" || uri.Scheme == "https"))
            {
                return GetNetworkStreamAsync(uri, progress);
            }

            return GetStreamFromUriCoreAsync(uri);
        }

        private static async Task<Stream> GetNetworkStreamAsync(Uri uri, IProgress<int> progress)
        {
            string cacheFileName = GetCacheFileName(uri);
            Stream cacheStream = await OpenTempFileStreamAsync(cacheFileName);
            if (cacheStream == null)
            {
                await DownloadToCacheFileAsync(uri, cacheFileName, progress);
                cacheStream = await OpenTempFileStreamAsync(cacheFileName);
            }
            progress.Report(100);
            return cacheStream;
        }
        private static async Task DownloadToCacheFileAsync(Uri uri, string fileName, IProgress<int> progress)
        {
            try
            {
                using HttpClient client = new();
                client.DefaultRequestHeaders.Add("User-Agent", ClientUserAgent);
                HttpRequestMessage request = new(HttpMethod.Get, uri);
                HttpResponseMessage response = await client.SendAsync(request);
                response.EnsureSuccessStatusCode();
                long length = response.Content.Headers.ContentLength ?? 0;
                using Stream responseStream = await response.Content.ReadAsStreamAsync();
                using Stream fileStream = await CreateTempFileStreamAsync(fileName);
                IProgress<long> absoluteProgress = null;
                if (progress != null)
                {
                    absoluteProgress =
                        new Progress<long>(bytesCopied =>
                        {
                            if (length > 0)
                            {
                                progress.Report((int)(100 * bytesCopied / length));
                            }
                            else
                            {
                                progress.Report(-1);
                            }
                        });
                }
                await responseStream.CopyToAsync(fileStream, absoluteProgress);
            }
            catch
            {
                DeleteTempFile(fileName);
                throw;
            }
        }

        private static Task<Stream> GetStreamFromUriCoreAsync(Uri uri)
        {
            if (uri.Scheme == PackUriHelper.UriSchemePack)
            {
                System.Windows.Resources.StreamResourceInfo sri = uri.Authority == "siteoforigin:,,,"
                    ? Application.GetRemoteStream(uri)
                    : Application.GetResourceStream(uri);

                if (sri != null)
                {
                    return Task.FromResult(sri.Stream);
                }

                throw new FileNotFoundException("Cannot find file with the specified URI");
            }

            if (uri.Scheme == Uri.UriSchemeFile)
            {
                return Task.FromResult<Stream>(File.OpenRead(uri.LocalPath));
            }

            throw new NotSupportedException("Only pack:, file:, http: and https: URIs are supported");
        }

        private static Task<Stream> OpenTempFileStreamAsync(string fileName)
        {
            if (!Directory.Exists(DownloadCacheLocation))
            {
                Directory.CreateDirectory(DownloadCacheLocation);
            }

            string path = Path.Combine(DownloadCacheLocation, fileName);
            Stream stream = null;
            DateTime expiration;
            try
            {
                expiration = File.GetLastWriteTime(path);

                if (DateTime.Now - expiration < DownloadCacheExpiration)
                {
                    stream = File.OpenRead(path);
                }

            }
            catch (FileNotFoundException)
            {
            }
            return Task.FromResult(stream);
        }

        private static Task<Stream> CreateTempFileStreamAsync(string fileName)
        {
            string path = Path.Combine(DownloadCacheLocation, fileName);
            Stream stream = File.OpenWrite(path);
            stream.SetLength(0);
            return Task.FromResult(stream);
        }

        private static void DeleteTempFile(string fileName)
        {
            string path = Path.Combine(DownloadCacheLocation, fileName);
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }

        private static string GetCacheFileName(Uri uri)
        {
            using SHA1 sha1 = SHA1.Create();
            byte[] bytes = Encoding.UTF8.GetBytes(uri.AbsoluteUri);
            byte[] hash = sha1.ComputeHash(bytes);
            return ToHex(hash);
        }

        private static string ToHex(byte[] bytes)
        {
            return bytes.Aggregate(
                new StringBuilder(),
                (sb, b) => sb.Append(b.ToString("X2")),
                sb => sb.ToString());
        }
    }
}