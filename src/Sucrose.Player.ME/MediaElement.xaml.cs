using System.IO;
using System.Text.RegularExpressions;
using System.Windows;
using SPMEEH = Sucrose.Player.ME.Event.Handler;
using SPMEHMEH = Sucrose.Player.ME.Helper.MediaElementHelper;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SMR = Sucrose.Memory.Readonly;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
#if NET48_OR_GREATER
using System.Net.Http;
#endif

namespace Sucrose.Player.ME
{
    /// <summary>
    /// Interaction logic for MediaElement.xaml
    /// </summary>
    public sealed partial class MediaElement : Window
    {
        public MediaElement()
        {
            InitializeComponent();

            Content = SPMEMI.MediaPlayer;

            string Source = @"https://prod-streaming-video-msn-com.akamaized.net/ba33094e-efb2-480f-ad77-1a103af156d2/3b215666-f0b7-4538-9ea2-0637d5ed2ea3.mp4";

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

                using HttpClient Client = new();
                using HttpResponseMessage Response = Client.GetAsync(Source).Result;
                using Stream Content = Response.Content.ReadAsStreamAsync().Result;
                using FileStream Stream = new(LocalSource, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                Content.CopyTo(Stream);

                SPMEMI.MediaPlayer.Source = new Uri(@Path.GetFullPath(LocalSource), UriKind.RelativeOrAbsolute);
#else
                SPMEMI.MediaPlayer.Source = new Uri(@Source, UriKind.RelativeOrAbsolute);
#endif
            }
            else
            {
                SPMEMI.MediaPlayer.Source = new Uri(@Path.GetFullPath(Source), UriKind.RelativeOrAbsolute);
            }

            SPMEMI.MediaPlayer.MediaOpened += SPMEEH.MediaPlayerOpened;
            SPMEMI.MediaPlayer.MediaFailed += SPMEEH.MediaPlayerFailed;
            SPMEMI.MediaPlayer.MediaEnded += SPMEEH.MediaPlayerEnded;

            Closing += (s, e) => SPMEMI.MediaPlayer.Close();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);

            SPMEHMEH.SetVolume(100);

            SPMEHMEH.Play();
        }

        public static bool IsUrl(string Address)
        {
            string Pattern = @"^(http|https|ftp)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?$";

            Regex Regex = new(Pattern, RegexOptions.IgnoreCase);

            return Regex.IsMatch(Address);
        }
    }
}