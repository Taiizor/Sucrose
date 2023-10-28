#if BROWSER

using Sucrose.WPF.CS;
using System.IO;
using SSIW = Sucrose.Signal.Interface.Websiter;
using SSMI = Sucrose.Signal.Manage.Internal;

namespace Sucrose.Shared.Signal.Services
{
    public static class WebsiterSignalService
    {
        public static async void Handler(object sender, FileSystemEventArgs e)
        {
            SSIW Data = await SSMI.WebsiterManager.FileRead<SSIW>(e.FullPath, new());

            Variables.State = true;
            Variables.Uri = Data.Url;
            Variables.Hook = Data.Hook;
        }
    }
}

#endif