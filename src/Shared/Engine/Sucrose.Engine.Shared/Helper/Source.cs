using System.IO;
using System.Net.Http;
using SMR = Sucrose.Memory.Readonly;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

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

        public static string GetYouTubeContent(string Video, string Playlist)
        {
            string Content = @"<html><head><meta charset=""UTF-8""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""X-UA-Compatible"" content=""ie=edge""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body{padding:0;margin:0;overflow:hidden}iframe{margin:0;position:absolute;left:0;top:0;overflow:hidden}#player{width:100%;height:100vh}</style></head><body><div id=""player""></div><script>var tag=document.createElement(""script"");tag.src=""https://www.youtube.com/iframe_api"";var firstScriptTag=document.getElementsByTagName(""script"")[0];firstScriptTag.parentNode.insertBefore(tag,firstScriptTag);var player,videoId=""{Video}"",playlistId=""{Playlist}"";function onYouTubeIframeAPIReady(){var e={height:""100%"",width:""100%"",playerVars:{autoplay:1,loop:1,controls:0,disablekb:1,modestbranding:1,rel:0,iv_load_policy:3,playsinline:0,cc_load_policy:0,version:3},events:{onStateChange:onPlayerStateChange,onReady:onPlayerReady,onError:onPlayerError}};videoId&&(e.videoId=videoId,e.playerVars.playlist=videoId),player=new YT.Player(""player"",e)}function onPlayerReady(e){playlistId&&(player.loadPlaylist({list:playlistId,listType:""playlist"",index:0,startSeconds:0,suggestedQuality:""highres""}),toggleFullScreen())}var shuffleMode=!0,prevIndex=-1,first=!0;function onPlayerStateChange(e){if(playlistId)if(e.data===YT.PlayerState.ENDED)if(shuffleMode){var l=player.getPlaylist();player.playVideoAt(Math.floor(Math.random()*l.length))}else{var t=player.getPlaylistIndex();t===prevIndex?(prevIndex=-1,player.playVideoAt(0)):prevIndex=t}else first&&-1===e.data&&(first=!1,shuffleMode)&&(l=player.getPlaylist(),player.playVideoAt(Math.floor(Math.random()*l.length)))}function onPlayerError(e){if(playlistId&&e.data){var l=player.getPlaylist(),t=player.getPlaylistIndex();t+1<l.length?player.playVideoAt(t+1):0<l.length&&player.playVideoAt(0)}}function playVideo(){player.playVideo()}function pauseVideo(){player.pauseVideo()}function setVolume(e){player.setVolume(e)}function setShuffle(e){shuffleMode=e}function setLoop(e){player.setLoop(e),e&&checkVideoEnded()&&playVideo()}function checkVideoEnded(){return player.getPlayerState()===YT.PlayerState.ENDED}function checkPlayingStatus(){return player.getPlayerState()===YT.PlayerState.PLAYING}function toggleFullScreen(){var e=document.getElementById(""player"");e.requestFullscreen?e.requestFullscreen():e.mozRequestFullScreen?e.mozRequestFullScreen():e.webkitRequestFullscreen?e.webkitRequestFullscreen():e.msRequestFullscreen&&e.msRequestFullscreen()}</script></body></html>";

            return Content.Replace("{Video}", Video).Replace("{Playlist}", Playlist);
        }

        public static void WriteVideoContent(string VideoContentPath, Uri Content)
        {
            WriteVideoContent(VideoContentPath, Content.ToString());
        }

        public static void WriteYouTubeContent(string YouTubeContentPath, string Video, string Playlist)
        {
            if (!Directory.Exists(Path.GetDirectoryName(YouTubeContentPath)))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(YouTubeContentPath));
            }

            File.WriteAllText(YouTubeContentPath, GetYouTubeContent(Video, Playlist));
        }

        public static void WriteVideoContent(string VideoContentPath, string Content)
        {
            if (!Directory.Exists(Path.GetDirectoryName(VideoContentPath)))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(VideoContentPath));
            }

            File.WriteAllText(VideoContentPath, GetVideoContent(Content));
        }

        public static string GetVideoContentPath()
        {
            return Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content, SMR.VideoContent);
        }

        public static string GetYouTubeContentPath()
        {
            return Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content, SMR.YouTubeContent);
        }

        public static Uri GetSource(Uri Source)
        {
            return GetSource(Source.ToString());
        }

        public static Uri GetSource(string Source, UriKind Kind = UriKind.RelativeOrAbsolute)
        {
            if (STSHV.IsUrl(Source))
            {
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
            }
            else
            {
                return new Uri(@Source, Kind);
            }
        }
    }
}