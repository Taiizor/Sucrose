using System.IO;
using System.Net.Http;
using SEST = Skylark.Enum.ScreenType;
using SMR = Sucrose.Memory.Readonly;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSMMS = Skylark.Struct.Monitor.MonitorStruct;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SWHSM = Skylark.Wing.Helper.ScreenManage;

namespace Sucrose.Shared.Engine.Helper
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

        public static string GetGifContent(Uri Source)
        {
            return GetGifContent(Source.ToString());
        }

        public static string GetGifContent(string Source)
        {
            string Content = @"<html><head><meta charset=""UTF-8""><meta http-equiv=""X-UA-Compatible"" content=""IE=edge,chrome=1""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body,html{margin:0;padding:0;height:100%;overflow:hidden}canvas{width:100%;height:100%;object-fit:none}</style></head><body><div><canvas id=""player""></canvas></div><div style=""display:none""><div><input id=""url"" type=""text"" value=""{Source}""><button id=""loadGIF"">Load</button><button id=""playpause"">Play/Pause</button></div><div><input id=""edgedetect"" type=""checkbox"">Edge Detect<input id=""invert"" type=""checkbox"">Invert Colours<input id=""loopmode"" type=""checkbox"" checked=""checked"">Loop Mode<input id=""grayscale"" type=""checkbox"">Grayscale</div><div><input id=""pixels"" type=""range"" min=""0"" max=""100"" step=""1"" value=""100"">Pixelate Amount</input></div></div><script>var gplayer=document.getElementById(""player"");function SucroseStretchMode(e){switch(e){case""None"":gplayer.style.objectFit=""none"";break;case""Fill"":gplayer.style.objectFit=""fill"";break;case""Uniform"":gplayer.style.objectFit=""contain"";break;case""UniformToFill"":gplayer.style.objectFit=""cover""}}function SucroseLoopMode(e){var t=document.getElementById(""loopmode"");t.checked=e;var r=new Event(""change"",{bubbles:e});t.dispatchEvent(r)}function SucrosePlayPause(){var e=document.getElementById(""playpause""),t=new Event(""click"");e.dispatchEvent(t)}!function(e){var t={};function r(n){if(t[n])return t[n].exports;var a=t[n]={i:n,l:!1,exports:{}};return e[n].call(a.exports,a,a.exports,r),a.l=!0,a.exports}r.m=e,r.c=t,r.d=function(e,t,n){r.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:n})},r.r=function(e){""undefined""!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:""Module""}),Object.defineProperty(e,""__esModule"",{value:!0})},r.t=function(e,t){if(1&t&&(e=r(e)),8&t)return e;if(4&t&&""object""==typeof e&&e&&e.__esModule)return e;var n=Object.create(null);if(r.r(n),Object.defineProperty(n,""default"",{enumerable:!0,value:e}),2&t&&""string""!=typeof e)for(var a in e)r.d(n,a,function(t){return e[t]}.bind(null,a));return n},r.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return r.d(t,""a"",t),t},r.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},r.p="""",r(r.s=0)}([function(e,t,r){""use strict"";r.r(t);var n,a=r(1),i=document.getElementById(""player""),o=i.getContext(""2d""),d=document.createElement(""canvas""),c=d.getContext(""2d""),u=document.createElement(""canvas""),s=u.getContext(""2d""),l=document.getElementById(""url"");function f(){var e=new XMLHttpRequest;e.open(""GET"",l.value,!0),e.responseType=""arraybuffer"",e.onload=function(t){var r=e.response;r&&(n=Object(a.parseGIF)(r),function(e){g=e,p=0,i.width=e[0].dims.width,i.height=e[0].dims.height,u.width=i.width,u.height=i.height,y||x()}(Object(a.decompressFrames)(n,!0)))},e.send(null)}document.getElementById(""loadGIF"").onclick=f,document.getElementById(""playpause"").onclick=x,document.getElementById(""edgedetect"").onchange=e=>{m=e.target.checked},document.getElementById(""grayscale"").onchange=e=>{w=e.target.checked},document.getElementById(""invert"").onchange=e=>{v=e.target.checked},document.getElementById(""loopmode"").onchange=e=>{b=e.target.checked,!y&&b&&x()},document.getElementById(""pixels"").onchange=e=>{B=e.target.value},f();var g,p,h,y=!1,m=!1,v=!1,b=!0,w=!1,B=100;function x(){(y=!y)&&E()}var k=function(e,t){for(var r=t.data,a=n.lsd.width,i=n.lsd.height,o=[-1,-1,-1,-1,8,-1,-1,-1,-1],d=Math.floor(1.5),c=0;c<i;c++)for(var u=0;u<a;u++){for(var s=0,l=0,f=0,g=0;g<3;g++)for(var p=0;p<3;p++){var h=c-d+g,y=u-d+p;if(h>=0&&h<i&&y>=0&&y<a){var m=4*(h*a+y),v=3*g+p;s+=e[m]*o[v],l+=e[m+1]*o[v],f+=e[m+2]*o[v]}}var b=4*(c*a+u);r[b]=s,r[b+1]=l,r[b+2]=f,r[b+3]=255}return t},I=function(e){for(var t=0;t<e.length;t+=4)e[t]=255-e[t],e[t+1]=255-e[t+1],e[t+2]=255-e[t+2],e[t+3]=255},S=function(e){for(var t=0;t<e.length;t+=4){var r=(e[t]+e[t+1]+e[t+2])/3;e[t]=r,e[t+1]=r,e[t+2]=r,e[t+3]=255}};function E(){var e=g[p],t=(new Date).getTime();2===e.disposalType&&s.clearRect(0,0,i.width,i.height),function(e){var t=e.dims;h&&t.width==h.width&&t.height==h.height||(d.width=t.width,d.height=t.height,h=c.createImageData(t.width,t.height)),h.data.set(e.patch),c.putImageData(h,0,0),s.drawImage(d,t.left,t.top)}(e),function(){var e=s.getImageData(0,0,u.width,u.height),t=s.createImageData(u.width,u.height);m&&(e=k(e.data,t)),v&&I(e.data),w&&S(e.data);var r=5+Math.floor(B/100*(i.width-5)),n=r*i.height/i.width;100!=B&&(o.mozImageSmoothingEnabled=!1,o.webkitImageSmoothingEnabled=!1,o.imageSmoothingEnabled=!1),o.putImageData(e,0,0),o.drawImage(i,0,0,i.width,i.height,0,0,r,n),o.drawImage(i,0,0,r,n,0,0,i.width,i.height)}(),++p>=g.length&&b?p=0:!b&&p>=g.length&&(y=!1,p=0);var r=(new Date).getTime()-t;y&&setTimeout((function(){requestAnimationFrame(E)}),Math.max(0,Math.floor(e.delay-r)))}},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.decompressFrames=t.decompressFrame=t.parseGIF=void 0;var n,a=(n=r(2))&&n.__esModule?n:{default:n},i=r(3),o=r(4),d=r(5),c=r(6);t.parseGIF=function(e){var t=new Uint8Array(e);return(0,i.parse)((0,o.buildStream)(t),a.default)};var u=function(e,t,r){if(e.image){var n=e.image,a=n.descriptor.width*n.descriptor.height,i=(0,c.lzw)(n.data.minCodeSize,n.data.blocks,a);n.descriptor.lct.interlaced&&(i=(0,d.deinterlace)(i,n.descriptor.width));var o={pixels:i,dims:{top:e.image.descriptor.top,left:e.image.descriptor.left,width:e.image.descriptor.width,height:e.image.descriptor.height}};return n.descriptor.lct&&n.descriptor.lct.exists?o.colorTable=n.lct:o.colorTable=t,e.gce&&(o.delay=10*(e.gce.delay||10),o.disposalType=e.gce.extras.disposal,e.gce.extras.transparentColorGiven&&(o.transparentIndex=e.gce.transparentColorIndex)),r&&(o.patch=function(e){for(var t=e.pixels.length,r=new Uint8ClampedArray(4*t),n=0;n<t;n++){var a=4*n,i=e.pixels[n],o=e.colorTable[i]||[0,0,0];r[a]=o[0],r[a+1]=o[1],r[a+2]=o[2],r[a+3]=i!==e.transparentIndex?255:0}return r}(o)),o}console.warn(""gif frame does not have associated image."")};t.decompressFrame=u;t.decompressFrames=function(e,t){return e.frames.filter((function(e){return e.image})).map((function(r){return u(r,e.gct,t)}))}},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.default=void 0;var n=r(3),a=r(4),i={blocks:function(e){for(var t=[],r=e.data.length,n=0,i=(0,a.readByte)()(e);0!==i;i=(0,a.readByte)()(e)){if(e.pos+i>=r){var o=r-e.pos;t.push((0,a.readBytes)(o)(e)),n+=o;break}t.push((0,a.readBytes)(i)(e)),n+=i}for(var d=new Uint8Array(n),c=0,u=0;u<t.length;u++)d.set(t[u],c),c+=t[u].length;return d}},o=(0,n.conditional)({gce:[{codes:(0,a.readBytes)(2)},{byteSize:(0,a.readByte)()},{extras:(0,a.readBits)({future:{index:0,length:3},disposal:{index:3,length:3},userInput:{index:6},transparentColorGiven:{index:7}})},{delay:(0,a.readUnsigned)(!0)},{transparentColorIndex:(0,a.readByte)()},{terminator:(0,a.readByte)()}]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&249===t[1]})),d=(0,n.conditional)({image:[{code:(0,a.readByte)()},{descriptor:[{left:(0,a.readUnsigned)(!0)},{top:(0,a.readUnsigned)(!0)},{width:(0,a.readUnsigned)(!0)},{height:(0,a.readUnsigned)(!0)},{lct:(0,a.readBits)({exists:{index:0},interlaced:{index:1},sort:{index:2},future:{index:3,length:2},size:{index:5,length:3}})}]},(0,n.conditional)({lct:(0,a.readArray)(3,(function(e,t,r){return Math.pow(2,r.descriptor.lct.size+1)}))},(function(e,t,r){return r.descriptor.lct.exists})),{data:[{minCodeSize:(0,a.readByte)()},i]}]},(function(e){return 44===(0,a.peekByte)()(e)})),c=(0,n.conditional)({text:[{codes:(0,a.readBytes)(2)},{blockSize:(0,a.readByte)()},{preData:function(e,t,r){return(0,a.readBytes)(r.text.blockSize)(e)}},i]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&1===t[1]})),u=(0,n.conditional)({application:[{codes:(0,a.readBytes)(2)},{blockSize:(0,a.readByte)()},{id:function(e,t,r){return(0,a.readString)(r.blockSize)(e)}},i]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&255===t[1]})),s=(0,n.conditional)({comment:[{codes:(0,a.readBytes)(2)},i]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&254===t[1]})),l=[{header:[{signature:(0,a.readString)(3)},{version:(0,a.readString)(3)}]},{lsd:[{width:(0,a.readUnsigned)(!0)},{height:(0,a.readUnsigned)(!0)},{gct:(0,a.readBits)({exists:{index:0},resolution:{index:1,length:3},sort:{index:4},size:{index:5,length:3}})},{backgroundColorIndex:(0,a.readByte)()},{pixelAspectRatio:(0,a.readByte)()}]},(0,n.conditional)({gct:(0,a.readArray)(3,(function(e,t){return Math.pow(2,t.lsd.gct.size+1)}))},(function(e,t){return t.lsd.gct.exists})),{frames:(0,n.loop)([o,u,s,d,c],(function(e){var t=(0,a.peekByte)()(e);return 33===t||44===t}))}];t.default=l},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.loop=t.conditional=t.parse=void 0;t.parse=function e(t,r){var n=arguments.length>2&&void 0!==arguments[2]?arguments[2]:{},a=arguments.length>3&&void 0!==arguments[3]?arguments[3]:n;if(Array.isArray(r))r.forEach((function(r){return e(t,r,n,a)}));else if(""function""==typeof r)r(t,n,a,e);else{var i=Object.keys(r)[0];Array.isArray(r[i])?(a[i]={},e(t,r[i],n,a[i])):a[i]=r[i](t,n,a,e)}return n};t.conditional=function(e,t){return function(r,n,a,i){t(r,n,a)&&i(r,e,n,a)}};t.loop=function(e,t){return function(r,n,a,i){for(var o=[];t(r,n,a);){var d={};i(r,e,n,d),o.push(d)}return o}}},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.readBits=t.readArray=t.readUnsigned=t.readString=t.peekBytes=t.readBytes=t.peekByte=t.readByte=t.buildStream=void 0;t.buildStream=function(e){return{data:e,pos:0}};var n=function(){return function(e){return e.data[e.pos++]}};t.readByte=n;t.peekByte=function(){var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:0;return function(t){return t.data[t.pos+e]}};var a=function(e){return function(t){return t.data.subarray(t.pos,t.pos+=e)}};t.readBytes=a;t.peekBytes=function(e){return function(t){return t.data.subarray(t.pos,t.pos+e)}};t.readString=function(e){return function(t){return Array.from(a(e)(t)).map((function(e){return String.fromCharCode(e)})).join("""")}};t.readUnsigned=function(e){return function(t){var r=a(2)(t);return e?(r[1]<<8)+r[0]:(r[0]<<8)+r[1]}};t.readArray=function(e,t){return function(r,n,i){for(var o=""function""==typeof t?t(r,n,i):t,d=a(e),c=new Array(o),u=0;u<o;u++)c[u]=d(r);return c}};t.readBits=function(e){return function(t){for(var r=function(e){return e.data[e.pos++]}(t),n=new Array(8),a=0;a<8;a++)n[7-a]=!!(r&1<<a);return Object.keys(e).reduce((function(t,r){var a=e[r];return a.length?t[r]=function(e,t,r){for(var n=0,a=0;a<r;a++)n+=e[t+a]&&Math.pow(2,r-a-1);return n}(n,a.index,a.length):t[r]=n[a.index],t}),{})}}},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.deinterlace=void 0;t.deinterlace=function(e,t){for(var r=new Array(e.length),n=e.length/t,a=function(n,a){var i=e.slice(a*t,(a+1)*t);r.splice.apply(r,[n*t,t].concat(i))},i=[0,4,2,1],o=[8,8,4,2],d=0,c=0;c<4;c++)for(var u=i[c];u<n;u+=o[c])a(u,d),d++;return r}},function(e,t,r){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.lzw=void 0;t.lzw=function(e,t,r){var n,a,i,o,d,c,u,s,l,f,g,p,h,y,m,v,b=4096,w=r,B=new Array(r),x=new Array(b),k=new Array(b),I=new Array(4097);for(d=(a=1<<(f=e))+1,n=a+2,u=-1,i=(1<<(o=f+1))-1,s=0;s<a;s++)x[s]=0,k[s]=s;for(g=p=h=y=m=v=0,l=0;l<w;){if(0===y){if(p<o){g+=t[v]<<p,p+=8,v++;continue}if(s=g&i,g>>=o,p-=o,s>n||s==d)break;if(s==a){i=(1<<(o=f+1))-1,n=a+2,u=-1;continue}if(-1==u){I[y++]=k[s],u=s,h=s;continue}for(c=s,s==n&&(I[y++]=h,s=u);s>a;)I[y++]=k[s],s=x[s];h=255&k[s],I[y++]=h,n<b&&(x[n]=u,k[n]=h,0==(++n&i)&&n<b&&(o++,i+=n)),u=c}y--,B[m++]=I[y],l++}for(l=m;l<w;l++)B[l]=0;return B}}]);</script></body></html>";

            return Content.Replace("{Source}", Source);
        }

        public static string GetVideoContent(Uri Source)
        {
            return GetVideoContent(Source.ToString());
        }

        public static string GetVideoContent(string Source)
        {
            return $"<html><head><meta charset=\"UTF-8\"><meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge,chrome=1\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Permissions-Policy\" content=\"interest-cohort=()\"><style>body{{padding:0;margin:0;overflow:hidden}}</style></head><body><video autoplay name=\"media\" src=\"{{Source}}\"></video></body></html>";
        }

        public static string GetYouTubeContent(string Video, string Playlist)
        {
            string Content = @"<html><head><meta charset=""UTF-8""><meta http-equiv=""X-UA-Compatible"" content=""IE=edge,chrome=1""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body{padding:0;margin:0;overflow:hidden}iframe{margin:0;position:absolute;left:0;top:0;overflow:hidden}#player{width:100%;height:100vh}</style></head><body><div id=""player""></div><script>var tag=document.createElement(""script"");tag.src=""https://www.youtube.com/iframe_api"";var firstScriptTag=document.getElementsByTagName(""script"")[0];firstScriptTag.parentNode.insertBefore(tag,firstScriptTag);var player,videoId=""{Video}"",playlistId=""{Playlist}"";function onYouTubeIframeAPIReady(){var e={height:""{Height}"",width:""{Width}"",playerVars:{autoplay:1,loop:1,controls:0,disablekb:1,modestbranding:1,fs:0,rel:0,iv_load_policy:3,playsinline:0,cc_load_policy:0,version:3,showinfo:0,suggestedQuality:""highres""},events:{onStateChange:onPlayerStateChange,onReady:onPlayerReady,onError:onPlayerError}};videoId&&(e.videoId=videoId,e.playerVars.playlist=videoId),player=new YT.Player(""player"",e)}function onPlayerReady(e){playlistId&&player.loadPlaylist({list:playlistId,listType:""playlist"",index:0,startSeconds:0,suggestedQuality:""highres""}),videoId&&!playlistId&&player.setLoop(!1),e.target.setPlaybackQuality(""highres""),e.target.setVolume(0),toggleFullScreen()}var shuffleMode=!0,prevIndex=-1,first=!0;function onPlayerStateChange(e){if(playlistId){if(e.data===YT.PlayerState.ENDED)if(shuffleMode){var t=player.getPlaylist();player.playVideoAt(Math.floor(Math.random()*t.length))}else{var l=player.getPlaylistIndex();l===prevIndex?(prevIndex=-1,player.playVideoAt(0)):prevIndex=l}else if(first&&-1===e.data&&(first=!1,shuffleMode)){t=player.getPlaylist();player.playVideoAt(Math.floor(Math.random()*t.length))}}else e.data===YT.PlayerState.ENDED&&(player.seekTo(0),player.playVideo())}function onPlayerError(e){if(playlistId&&e.data){var t=player.getPlaylist(),l=player.getPlaylistIndex();l+1<t.length?player.playVideoAt(l+1):0<t.length&&player.playVideoAt(0)}}function playFirst(){-1!=player.getPlayerState()&&3!=player.getPlayerState()||playVideo()}function playVideo(){player.playVideo()}function pauseVideo(){player.pauseVideo()}function setVolume(e){player.setVolume(e)}function setShuffle(e){shuffleMode=e}function setLoop(e){e&&videoId&&!playlistId&&(e=!1),player.setLoop(e),e&&checkVideoEnded()&&playVideo()}function checkVideoEnded(){return player.getPlayerState()===YT.PlayerState.ENDED}function checkPlayingStatus(){return player.getPlayerState()===YT.PlayerState.PLAYING}function toggleFullScreen(){var e=document.getElementById(""player"");e.requestFullscreen?e.requestFullscreen():e.mozRequestFullScreen?e.mozRequestFullScreen():e.webkitRequestFullscreen?e.webkitRequestFullscreen():e.msRequestFullscreen&&e.msRequestFullscreen()}</script></body></html>";

            SEST Screen = SSEHD.GetScreenType();
            SSDEDT Display = SSEHD.GetDisplayType();

            switch (Display)
            {
                case SSDEDT.Expand:
                    SSMMS EMonitor = SWHSM.OwnerScreen(SSEHD.GetExpandScreenType()); Content = Screen switch
                    {
                        SEST.WorkingArea => Content.Replace("{Height}", $"{EMonitor.rcWork.Height}").Replace("{Width}", $"{EMonitor.rcWork.Width}"),
                        _ => Content.Replace("{Height}", $"{EMonitor.rcMonitor.Height}").Replace("{Width}", $"{EMonitor.rcMonitor.Width}"),
                    };
                    break;
                case SSDEDT.Duplicate:
                    SSMMS DMonitor = SWHSM.OwnerScreen(0);
                    Content = Screen switch
                    {
                        SEST.WorkingArea => Content.Replace("{Height}", $"{DMonitor.rcWork.Height}").Replace("{Width}", $"{DMonitor.rcWork.Width}"),
                        _ => Content.Replace("{Height}", $"{DMonitor.rcMonitor.Height}").Replace("{Width}", $"{DMonitor.rcMonitor.Width}"),
                    };
                    break;
                default:
                    SSMMS SMonitor = SWHSM.OwnerScreen(SSEHD.GetScreenIndex());
                    Content = Screen switch
                    {
                        SEST.WorkingArea => Content.Replace("{Height}", $"{SMonitor.rcWork.Height}").Replace("{Width}", $"{SMonitor.rcWork.Width}"),
                        _ => Content.Replace("{Height}", $"{SMonitor.rcMonitor.Height}").Replace("{Width}", $"{SMonitor.rcMonitor.Width}"),
                    };
                    break;
            }

            return Content.Replace("{Video}", Video).Replace("{Playlist}", Playlist);
        }

        public static void WriteGifContent(string GifContentPath, Uri Content)
        {
            WriteGifContent(GifContentPath, Content.ToString());
        }

        public static void WriteGifContent(string GifContentPath, string Content)
        {
            if (!Directory.Exists(Path.GetDirectoryName(GifContentPath)))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(GifContentPath));
            }

            File.WriteAllText(GifContentPath, GetGifContent(Content));
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

        public static string GetGifContentPath()
        {
            return Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content, SMR.GifContent);
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
            if (SSTHV.IsUrl(Source))
            {
                string CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Content);

                if (!Directory.Exists(CachePath))
                {
                    Directory.CreateDirectory(CachePath);
                }

                //string LocalSource = Path.Combine(CachePath, Path.ChangeExtension(Path.GetRandomFileName(), Path.GetExtension(Source)));
                string LocalSource = Path.Combine(CachePath, $"{SSECCE.TextToMD5(Source)}{Path.GetExtension(Source)}");

                if (File.Exists(LocalSource))
                {
                    return new Uri(LocalSource, Kind);
                }
                else
                {
                    using HttpClient Client = new()
                    {
                        Timeout = Timeout.InfiniteTimeSpan
                    };

                    using HttpResponseMessage Response = Client.GetAsync(Source).Result;
                    using Stream Content = Response.Content.ReadAsStreamAsync().Result;
                    using FileStream Stream = new(LocalSource, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                    Content.CopyTo(Stream);

                    Content.Dispose();
                    Stream.Dispose();

                    return new Uri(Path.GetFullPath(LocalSource), Kind);
                }
            }
            else
            {
                return new Uri(Source, Kind);
            }
        }
    }
}