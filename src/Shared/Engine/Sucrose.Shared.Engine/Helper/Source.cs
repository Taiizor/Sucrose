using System.IO;
using System.Net.Http;
using SEDST = Skylark.Enum.DisplayScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMR = Sucrose.Memory.Readonly;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSMMS = Skylark.Struct.Monitor.MonitorStruct;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SWHSM = Skylark.Wing.Helper.ScreenManage;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Source
    {
        public static string GetVideoStyle()
        {
            return "var style=document.createElement(\"style\");style.textContent=\"video::-webkit-media-controls,video::-webkit-media-controls-timeline,video::-webkit-media-controls-mute-button,video::-webkit-media-controls-play-button,video::-webkit-media-controls-volume-slider,video::-webkit-media-controls-overflow-button,video::-webkit-media-controls-fullscreen-button,video::-webkit-media-controls-current-time-display,video::-webkit-media-controls-overflow-menu-button,video::-webkit-media-controls-time-remaining-display,video::-webkit-media-controls-toggle-closed-captions-button {display: none !important;}\",document.head.appendChild(style);";
        }

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
            string Content = @"<html><head><meta charset=""UTF-8""><meta http-equiv=""X-UA-Compatible"" content=""IE=edge,chrome=1""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body,html{margin:0;padding:0;height:100%;overflow:hidden}canvas{width:100%;height:100%;object-fit:none}</style></head><body><div><canvas id=""player""></canvas></div><div style=""display:none""><div><input id=""url"" type=""text"" value=""{Source}""><button id=""loadGIF"">Load</button><button id=""play"">Play</button><button id=""pause"">Pause</button><button id=""playpause"">Play/Pause</button></div><div><input id=""edgedetect"" type=""checkbox"">Edge Detect<input id=""invert"" type=""checkbox"">Invert Colours<input id=""loopmode"" type=""checkbox"" checked=""checked"">Loop Mode<input id=""grayscale"" type=""checkbox"">Grayscale</div><div><input id=""pixels"" type=""range"" min=""0"" max=""100"" step=""1"" value=""100"">Pixelate Amount</div></div><script>var gplayer=document.getElementById(""player"");function SucroseStretchMode(e){switch(e){case""None"":gplayer.style.objectFit=""none"";break;case""Fill"":gplayer.style.objectFit=""fill"";break;case""Uniform"":gplayer.style.objectFit=""contain"";break;case""UniformToFill"":gplayer.style.objectFit=""cover""}}function SucroseLoopMode(e){var t=document.getElementById(""loopmode"");t.checked=e;var n=new Event(""change"",{bubbles:e});t.dispatchEvent(n)}function SucrosePlay(){var e=document.getElementById(""play""),t=new Event(""click"");e.dispatchEvent(t)}function SucrosePause(){var e=document.getElementById(""pause""),t=new Event(""click"");e.dispatchEvent(t)}function SucrosePlayPause(){var e=document.getElementById(""playpause""),t=new Event(""click"");e.dispatchEvent(t)}!function(e){var t={};function n(r){if(t[r])return t[r].exports;var a=t[r]={i:r,l:!1,exports:{}};return e[r].call(a.exports,a,a.exports,n),a.l=!0,a.exports}n.m=e,n.c=t,n.d=function(e,t,r){n.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:r})},n.r=function(e){""undefined""!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:""Module""}),Object.defineProperty(e,""__esModule"",{value:!0})},n.t=function(e,t){if(1&t&&(e=n(e)),8&t)return e;if(4&t&&""object""==typeof e&&e&&e.__esModule)return e;var r=Object.create(null);if(n.r(r),Object.defineProperty(r,""default"",{enumerable:!0,value:e}),2&t&&""string""!=typeof e)for(var a in e)n.d(r,a,function(t){return e[t]}.bind(null,a));return r},n.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return n.d(t,""a"",t),t},n.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},n.p="""",n(n.s=0)}([function(e,t,n){""use strict"";n.r(t);var r,a=n(1),o=document.getElementById(""player""),i=o.getContext(""2d""),d=document.createElement(""canvas""),c=d.getContext(""2d""),u=document.createElement(""canvas""),s=u.getContext(""2d""),l=document.getElementById(""url"");function f(){var e=new XMLHttpRequest;e.open(""GET"",l.value,!0),e.responseType=""arraybuffer"",e.onload=function(t){var n=e.response;n&&(r=Object(a.parseGIF)(n),function(e){g=e,p=0,o.width=e[0].dims.width,o.height=e[0].dims.height,u.width=o.width,u.height=o.height,m&&!y&&k()}(Object(a.decompressFrames)(r,!0)))},e.send(null)}document.getElementById(""play"").onclick=function(){m||(m=y=!0,j())},document.getElementById(""pause"").onclick=function(){m&&(m=y=!1)},document.getElementById(""loadGIF"").onclick=f,document.getElementById(""playpause"").onclick=k,document.getElementById(""edgedetect"").onchange=e=>{v=e.target.checked},document.getElementById(""grayscale"").onchange=e=>{B=e.target.checked},document.getElementById(""invert"").onchange=e=>{b=e.target.checked},document.getElementById(""loopmode"").onchange=e=>{w=e.target.checked,m&&!y&&w&&k()},document.getElementById(""pixels"").onchange=e=>{x=e.target.value},f();var g,p,h,y=!1,m=!1,v=!1,b=!1,w=!0,B=!1,x=100;function k(){m=y=!y,y&&j()}var I=function(e,t){for(var n=t.data,a=r.lsd.width,o=r.lsd.height,i=[-1,-1,-1,-1,8,-1,-1,-1,-1],d=Math.floor(1.5),c=0;c<o;c++)for(var u=0;u<a;u++){for(var s=0,l=0,f=0,g=0;g<3;g++)for(var p=0;p<3;p++){var h=c-d+g,y=u-d+p;if(h>=0&&h<o&&y>=0&&y<a){var m=4*(h*a+y),v=3*g+p;s+=e[m]*i[v],l+=e[m+1]*i[v],f+=e[m+2]*i[v]}}var b=4*(c*a+u);n[b]=s,n[b+1]=l,n[b+2]=f,n[b+3]=255}return t},E=function(e){for(var t=0;t<e.length;t+=4)e[t]=255-e[t],e[t+1]=255-e[t+1],e[t+2]=255-e[t+2],e[t+3]=255},S=function(e){for(var t=0;t<e.length;t+=4){var n=(e[t]+e[t+1]+e[t+2])/3;e[t]=n,e[t+1]=n,e[t+2]=n,e[t+3]=255}};function j(){var e=g[p],t=(new Date).getTime();2===e.disposalType&&s.clearRect(0,0,o.width,o.height),function(e){var t=e.dims;h&&t.width==h.width&&t.height==h.height||(d.width=t.width,d.height=t.height,h=c.createImageData(t.width,t.height)),h.data.set(e.patch),c.putImageData(h,0,0),s.drawImage(d,t.left,t.top)}(e),function(){var e=s.getImageData(0,0,u.width,u.height),t=s.createImageData(u.width,u.height);v&&(e=I(e.data,t)),b&&E(e.data),B&&S(e.data);var n=5+Math.floor(x/100*(o.width-5)),r=n*o.height/o.width;100!=x&&(i.mozImageSmoothingEnabled=!1,i.webkitImageSmoothingEnabled=!1,i.imageSmoothingEnabled=!1),i.putImageData(e,0,0),i.drawImage(o,0,0,o.width,o.height,0,0,n,r),i.drawImage(o,0,0,n,r,0,0,o.width,o.height)}(),++p>=g.length&&w?p=0:!w&&p>=g.length&&(y=!1,p=0);var n=(new Date).getTime()-t;y&&m&&setTimeout((function(){requestAnimationFrame(j)}),Math.max(0,Math.floor(e.delay-n)))}},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.decompressFrames=t.decompressFrame=t.parseGIF=void 0;var r,a=(r=n(2))&&r.__esModule?r:{default:r},o=n(3),i=n(4),d=n(5),c=n(6);t.parseGIF=function(e){var t=new Uint8Array(e);return(0,o.parse)((0,i.buildStream)(t),a.default)};var u=function(e,t,n){if(e.image){var r=e.image,a=r.descriptor.width*r.descriptor.height,o=(0,c.lzw)(r.data.minCodeSize,r.data.blocks,a);r.descriptor.lct.interlaced&&(o=(0,d.deinterlace)(o,r.descriptor.width));var i={pixels:o,dims:{top:e.image.descriptor.top,left:e.image.descriptor.left,width:e.image.descriptor.width,height:e.image.descriptor.height}};return r.descriptor.lct&&r.descriptor.lct.exists?i.colorTable=r.lct:i.colorTable=t,e.gce&&(i.delay=10*(e.gce.delay||10),i.disposalType=e.gce.extras.disposal,e.gce.extras.transparentColorGiven&&(i.transparentIndex=e.gce.transparentColorIndex)),n&&(i.patch=function(e){for(var t=e.pixels.length,n=new Uint8ClampedArray(4*t),r=0;r<t;r++){var a=4*r,o=e.pixels[r],i=e.colorTable[o]||[0,0,0];n[a]=i[0],n[a+1]=i[1],n[a+2]=i[2],n[a+3]=o!==e.transparentIndex?255:0}return n}(i)),i}console.warn(""gif frame does not have associated image."")};t.decompressFrame=u;t.decompressFrames=function(e,t){return e.frames.filter((function(e){return e.image})).map((function(n){return u(n,e.gct,t)}))}},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.default=void 0;var r=n(3),a=n(4),o={blocks:function(e){for(var t=[],n=e.data.length,r=0,o=(0,a.readByte)()(e);0!==o;o=(0,a.readByte)()(e)){if(e.pos+o>=n){var i=n-e.pos;t.push((0,a.readBytes)(i)(e)),r+=i;break}t.push((0,a.readBytes)(o)(e)),r+=o}for(var d=new Uint8Array(r),c=0,u=0;u<t.length;u++)d.set(t[u],c),c+=t[u].length;return d}},i=(0,r.conditional)({gce:[{codes:(0,a.readBytes)(2)},{byteSize:(0,a.readByte)()},{extras:(0,a.readBits)({future:{index:0,length:3},disposal:{index:3,length:3},userInput:{index:6},transparentColorGiven:{index:7}})},{delay:(0,a.readUnsigned)(!0)},{transparentColorIndex:(0,a.readByte)()},{terminator:(0,a.readByte)()}]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&249===t[1]})),d=(0,r.conditional)({image:[{code:(0,a.readByte)()},{descriptor:[{left:(0,a.readUnsigned)(!0)},{top:(0,a.readUnsigned)(!0)},{width:(0,a.readUnsigned)(!0)},{height:(0,a.readUnsigned)(!0)},{lct:(0,a.readBits)({exists:{index:0},interlaced:{index:1},sort:{index:2},future:{index:3,length:2},size:{index:5,length:3}})}]},(0,r.conditional)({lct:(0,a.readArray)(3,(function(e,t,n){return Math.pow(2,n.descriptor.lct.size+1)}))},(function(e,t,n){return n.descriptor.lct.exists})),{data:[{minCodeSize:(0,a.readByte)()},o]}]},(function(e){return 44===(0,a.peekByte)()(e)})),c=(0,r.conditional)({text:[{codes:(0,a.readBytes)(2)},{blockSize:(0,a.readByte)()},{preData:function(e,t,n){return(0,a.readBytes)(n.text.blockSize)(e)}},o]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&1===t[1]})),u=(0,r.conditional)({application:[{codes:(0,a.readBytes)(2)},{blockSize:(0,a.readByte)()},{id:function(e,t,n){return(0,a.readString)(n.blockSize)(e)}},o]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&255===t[1]})),s=(0,r.conditional)({comment:[{codes:(0,a.readBytes)(2)},o]},(function(e){var t=(0,a.peekBytes)(2)(e);return 33===t[0]&&254===t[1]})),l=[{header:[{signature:(0,a.readString)(3)},{version:(0,a.readString)(3)}]},{lsd:[{width:(0,a.readUnsigned)(!0)},{height:(0,a.readUnsigned)(!0)},{gct:(0,a.readBits)({exists:{index:0},resolution:{index:1,length:3},sort:{index:4},size:{index:5,length:3}})},{backgroundColorIndex:(0,a.readByte)()},{pixelAspectRatio:(0,a.readByte)()}]},(0,r.conditional)({gct:(0,a.readArray)(3,(function(e,t){return Math.pow(2,t.lsd.gct.size+1)}))},(function(e,t){return t.lsd.gct.exists})),{frames:(0,r.loop)([i,u,s,d,c],(function(e){var t=(0,a.peekByte)()(e);return 33===t||44===t}))}];t.default=l},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.loop=t.conditional=t.parse=void 0;t.parse=function e(t,n){var r=arguments.length>2&&void 0!==arguments[2]?arguments[2]:{},a=arguments.length>3&&void 0!==arguments[3]?arguments[3]:r;if(Array.isArray(n))n.forEach((function(n){return e(t,n,r,a)}));else if(""function""==typeof n)n(t,r,a,e);else{var o=Object.keys(n)[0];Array.isArray(n[o])?(a[o]={},e(t,n[o],r,a[o])):a[o]=n[o](t,r,a,e)}return r};t.conditional=function(e,t){return function(n,r,a,o){t(n,r,a)&&o(n,e,r,a)}};t.loop=function(e,t){return function(n,r,a,o){for(var i=[];t(n,r,a);){var d={};o(n,e,r,d),i.push(d)}return i}}},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.readBits=t.readArray=t.readUnsigned=t.readString=t.peekBytes=t.readBytes=t.peekByte=t.readByte=t.buildStream=void 0;t.buildStream=function(e){return{data:e,pos:0}};var r=function(){return function(e){return e.data[e.pos++]}};t.readByte=r;t.peekByte=function(){var e=arguments.length>0&&void 0!==arguments[0]?arguments[0]:0;return function(t){return t.data[t.pos+e]}};var a=function(e){return function(t){return t.data.subarray(t.pos,t.pos+=e)}};t.readBytes=a;t.peekBytes=function(e){return function(t){return t.data.subarray(t.pos,t.pos+e)}};t.readString=function(e){return function(t){return Array.from(a(e)(t)).map((function(e){return String.fromCharCode(e)})).join("""")}};t.readUnsigned=function(e){return function(t){var n=a(2)(t);return e?(n[1]<<8)+n[0]:(n[0]<<8)+n[1]}};t.readArray=function(e,t){return function(n,r,o){for(var i=""function""==typeof t?t(n,r,o):t,d=a(e),c=new Array(i),u=0;u<i;u++)c[u]=d(n);return c}};t.readBits=function(e){return function(t){for(var n=function(e){return e.data[e.pos++]}(t),r=new Array(8),a=0;a<8;a++)r[7-a]=!!(n&1<<a);return Object.keys(e).reduce((function(t,n){var a=e[n];return a.length?t[n]=function(e,t,n){for(var r=0,a=0;a<n;a++)r+=e[t+a]&&Math.pow(2,n-a-1);return r}(r,a.index,a.length):t[n]=r[a.index],t}),{})}}},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.deinterlace=void 0;t.deinterlace=function(e,t){for(var n=new Array(e.length),r=e.length/t,a=function(r,a){var o=e.slice(a*t,(a+1)*t);n.splice.apply(n,[r*t,t].concat(o))},o=[0,4,2,1],i=[8,8,4,2],d=0,c=0;c<4;c++)for(var u=o[c];u<r;u+=i[c])a(u,d),d++;return n}},function(e,t,n){""use strict"";Object.defineProperty(t,""__esModule"",{value:!0}),t.lzw=void 0;t.lzw=function(e,t,n){var r,a,o,i,d,c,u,s,l,f,g,p,h,y,m,v,b=4096,w=n,B=new Array(n),x=new Array(b),k=new Array(b),I=new Array(4097);for(d=(a=1<<(f=e))+1,r=a+2,u=-1,o=(1<<(i=f+1))-1,s=0;s<a;s++)x[s]=0,k[s]=s;for(g=p=h=y=m=v=0,l=0;l<w;){if(0===y){if(p<i){g+=t[v]<<p,p+=8,v++;continue}if(s=g&o,g>>=i,p-=i,s>r||s==d)break;if(s==a){o=(1<<(i=f+1))-1,r=a+2,u=-1;continue}if(-1==u){I[y++]=k[s],u=s,h=s;continue}for(c=s,s==r&&(I[y++]=h,s=u);s>a;)I[y++]=k[s],s=x[s];h=255&k[s],I[y++]=h,r<b&&(x[r]=u,k[r]=h,!(++r&o)&&r<b&&(i++,o+=r)),u=c}y--,B[m++]=I[y],l++}for(l=m;l<w;l++)B[l]=0;return B}}]);</script></body></html>";

            return Content.Replace("{Source}", Source);
        }

        public static string GetVideoContent(Uri Source)
        {
            return GetVideoContent(Source.ToString());
        }

        public static string GetVideoContent(string Source)
        {
            string Content = @"<html><head><meta charset=""UTF-8""><meta http-equiv=""X-UA-Compatible"" content=""IE=edge,chrome=1""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body{padding:0;margin:0;overflow:hidden}video::-webkit-media-controls,video::-webkit-media-controls-current-time-display,video::-webkit-media-controls-fullscreen-button,video::-webkit-media-controls-mute-button,video::-webkit-media-controls-overflow-button,video::-webkit-media-controls-overflow-menu-button,video::-webkit-media-controls-play-button,video::-webkit-media-controls-time-remaining-display,video::-webkit-media-controls-timeline,video::-webkit-media-controls-toggle-closed-captions-button,video::-webkit-media-controls-volume-slider{display:none!important}</style></head><body><video autoplay name=""media"" src=""{Source}""></video></body></html>";

            return Content.Replace("{Source}", Source);
        }

        public static string GetYouTubeContent(string Video, string Playlist)
        {
            string Content = @"<html><head><meta charset=""UTF-8""><meta http-equiv=""X-UA-Compatible"" content=""IE=edge,chrome=1""><meta name=""viewport"" content=""width=device-width,initial-scale=1""><meta http-equiv=""Permissions-Policy"" content=""interest-cohort=()""><style>body{padding:0;margin:0;overflow:hidden}iframe{margin:0;position:absolute;left:0;top:0;overflow:hidden}#player{width:100%;height:100vh}</style></head><body><div id=""player""></div><script>var tag=document.createElement(""script"");tag.src=""https://www.youtube.com/iframe_api"";var firstScriptTag=document.getElementsByTagName(""script"")[0];firstScriptTag.parentNode.insertBefore(tag,firstScriptTag);var player,videoId=""{Video}"",playlistId=""{Playlist}"";function onYouTubeIframeAPIReady(){var e={height:""{Height}"",width:""{Width}"",playerVars:{autoplay:1,loop:1,controls:0,disablekb:1,modestbranding:1,fs:0,rel:0,iv_load_policy:3,playsinline:0,cc_load_policy:0,version:3,showinfo:0,suggestedQuality:""highres""},events:{onStateChange:onPlayerStateChange,onReady:onPlayerReady,onError:onPlayerError}};videoId&&(e.videoId=videoId,e.playerVars.playlist=videoId),player=new YT.Player(""player"",e)}function onPlayerReady(e){playlistId&&player.loadPlaylist({list:playlistId,listType:""playlist"",index:0,startSeconds:0,suggestedQuality:""highres""}),videoId&&!playlistId&&player.setLoop(!1),e.target.setPlaybackQuality(""highres""),e.target.setVolume(0),toggleFullScreen()}var shuffleMode=!0,prevIndex=-1,first=!0;function onPlayerStateChange(e){if(playlistId){if(e.data===YT.PlayerState.ENDED)if(shuffleMode){var t=player.getPlaylist();player.playVideoAt(Math.floor(Math.random()*t.length))}else{var l=player.getPlaylistIndex();l===prevIndex?(prevIndex=-1,player.playVideoAt(0)):prevIndex=l}else if(first&&-1===e.data&&(first=!1,shuffleMode)){t=player.getPlaylist();player.playVideoAt(Math.floor(Math.random()*t.length))}}else e.data===YT.PlayerState.ENDED&&(player.seekTo(0),player.playVideo())}function onPlayerError(e){if(playlistId&&e.data){var t=player.getPlaylist(),l=player.getPlaylistIndex();l+1<t.length?player.playVideoAt(l+1):0<t.length&&player.playVideoAt(0)}}function playFirst(){-1!=player.getPlayerState()&&3!=player.getPlayerState()||playVideo()}function playVideo(){player.playVideo()}function pauseVideo(){player.pauseVideo()}function setVolume(e){player.setVolume(e)}function setShuffle(e){shuffleMode=e}function setLoop(e){e&&videoId&&!playlistId&&(e=!1),player.setLoop(e),e&&checkVideoEnded()&&playVideo()}function checkVideoEnded(){return player.getPlayerState()===YT.PlayerState.ENDED}function checkPlayingStatus(){return player.getPlayerState()===YT.PlayerState.PLAYING}function toggleFullScreen(){var e=document.getElementById(""player"");e.requestFullscreen?e.requestFullscreen():e.mozRequestFullScreen?e.mozRequestFullScreen():e.webkitRequestFullscreen?e.webkitRequestFullscreen():e.msRequestFullscreen&&e.msRequestFullscreen()}</script></body></html>";

            SEST Screen = SSEHD.GetScreenType();
            SEDST Display = SSEHD.GetDisplayScreenType();

            switch (Display)
            {
                case SEDST.SpanAcross:
                    SSMMS EMonitor = SWHSM.OwnerScreen(SSEHD.GetExpandScreenType()); Content = Screen switch
                    {
                        SEST.WorkingArea => Content.Replace("{Height}", $"{EMonitor.rcWork.Height}").Replace("{Width}", $"{EMonitor.rcWork.Width}"),
                        _ => Content.Replace("{Height}", $"{EMonitor.rcMonitor.Height}").Replace("{Width}", $"{EMonitor.rcMonitor.Width}"),
                    };
                    break;
                case SEDST.SameDuplicate:
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