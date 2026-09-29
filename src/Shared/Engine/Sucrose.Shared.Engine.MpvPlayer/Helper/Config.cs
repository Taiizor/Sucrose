using System.IO;
using System.Text.RegularExpressions;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SSDEPT = Sucrose.Shared.Dependency.Enum.ProxyType;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMPMI = Sucrose.Shared.Engine.MpvPlayer.Manage.Internal;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHR = Sucrose.Shared.Space.Helper.Regexer;

namespace Sucrose.Shared.Engine.MpvPlayer.Helper
{
    internal static class Config
    {
        public static void Start()
        {
            if (!Directory.Exists(SSEMPMI.MpvPath))
            {
                Directory.CreateDirectory(SSEMPMI.MpvPath);
            }

            SSEMPMI.MpvConfig = Path.Combine(SSEMPMI.MpvPath, SMMRC.uMpvPlayerConfig);

            if (!File.Exists(SSEMPMI.MpvConfig))
            {
                SSEMPMI.MpvConfig = Path.Combine(SSEMPMI.MpvPath, SMMRC.MpvPlayerConfig);

                string Content = string.Join(Environment.NewLine, SSEMI.MpvConfig);

                if (SMME.StayAwake)
                {
                    Content = SSSHR.Replace(Content, @"^stop-screensaver=.*$", "stop-screensaver=always", RegexOptions.Multiline);
                }
                else
                {
                    Content = SSSHR.Replace(Content, @"^stop-screensaver=.*$", "stop-screensaver=no", RegexOptions.Multiline);
                }

                if (SMME.HardwareAcceleration)
                {
                    Content = SSSHR.Replace(Content, @"^hwdec=.*$", "hwdec=auto-safe", RegexOptions.Multiline);
                }
                else
                {
                    Content = SSSHR.Replace(Content, @"^hwdec=.*$", "hwdec=no", RegexOptions.Multiline);
                }

                if (SMME.ProxyEnabled && !string.IsNullOrEmpty(SMME.ProxyServer) && SMME.ProxyPort > 0)
                {
                    string proxyUrl = BuildProxyUrl();
                    
                    if (!string.IsNullOrEmpty(proxyUrl))
                    {
                        Content += Environment.NewLine + Environment.NewLine;
                        Content += "# Proxy Settings #" + Environment.NewLine;
                        Content += $"http-proxy={proxyUrl}" + Environment.NewLine;
                        Content += "# Proxy Settings #";
                    }
                }

                SSSHF.WriteStream(SSEMPMI.MpvConfig, Content);
            }
        }

        private static string BuildProxyUrl()
        {
            if (!SMME.ProxyEnabled || string.IsNullOrEmpty(SMME.ProxyServer) || SMME.ProxyPort <= 0)
            {
                return string.Empty;
            }

            string protocol = SMME.ProxyType switch
            {
                SSDEPT.HTTP => "http",
                SSDEPT.HTTPS => "https",
                SSDEPT.SOCKS5 => "socks5",
                _ => "http"
            };

            string auth = string.Empty;
            if (!string.IsNullOrEmpty(SMME.ProxyUsername))
            {
                auth = string.IsNullOrEmpty(SMME.ProxyPassword) 
                    ? $"{SMME.ProxyUsername}@" 
                    : $"{SMME.ProxyUsername}:{SMME.ProxyPassword}@";
            }

            return $"{protocol}://{auth}{SMME.ProxyServer}:{SMME.ProxyPort}";
        }
    }
}