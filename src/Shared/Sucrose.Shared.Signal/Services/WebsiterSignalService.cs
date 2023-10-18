#if BROWSER

using System.IO;
using SSIW = Sucrose.Signal.Interface.Websiter;
using SSMI = Sucrose.Signal.Manage.Internal;
using Sucrose.WPF.CS;

namespace Sucrose.Shared.Signal.Services
{
    public static class WebsiterSignalService
    {
        public static void Handler(object sender, FileSystemEventArgs e)
        {
            SSIW Data = SSMI.WebsiterManager.FileRead<SSIW>(e.FullPath, new());

            Variables.State = true;
            Variables.Uri = Data.Url;
            Variables.Hook = Data.Hook;
        }
    }
}

#endif