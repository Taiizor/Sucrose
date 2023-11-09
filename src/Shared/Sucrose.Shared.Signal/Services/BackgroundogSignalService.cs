#if LIVE_WEBVIEW || LIVE_CEFSHARP

using Newtonsoft.Json;
using System.IO;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSIB = Sucrose.Signal.Interface.Backgroundog;
using SSMI = Sucrose.Signal.Manage.Internal;

namespace Sucrose.Shared.Signal.Services
{
    public static class BackgroundogSignalService
    {
        public static async void Handler(object sender, FileSystemEventArgs e)
        {
            if (e != null)
            {
                SSIB Data = await SSMI.BackgroundogManager.FileRead<SSIB>(e.FullPath, new());

                if (Data != null)
                {
                    if (Data.Cpu != null)
                    {
                        SSEMI.CpuData = JsonConvert.SerializeObject(Data.Cpu, Formatting.Indented);
                    }

                    if (Data.Bios != null)
                    {
                        SSEMI.BiosData = JsonConvert.SerializeObject(Data.Bios, Formatting.Indented);
                    }

                    if (Data.Date != null)
                    {
                        SSEMI.DateData = JsonConvert.SerializeObject(Data.Date, Formatting.Indented);
                    }

                    if (Data.Audio != null)
                    {
                        SSEMI.AudioData = JsonConvert.SerializeObject(Data.Audio, Formatting.Indented);
                    }

                    if (Data.Memory != null)
                    {
                        SSEMI.MemoryData = JsonConvert.SerializeObject(Data.Memory, Formatting.Indented);
                    }

                    if (Data.Battery != null)
                    {
                        SSEMI.BatteryData = JsonConvert.SerializeObject(Data.Battery, Formatting.Indented);
                    }

                    if (Data.Graphic != null)
                    {
                        SSEMI.GraphicData = JsonConvert.SerializeObject(Data.Graphic, Formatting.Indented);
                    }

                    if (Data.Network != null)
                    {
                        SSEMI.NetworkData = JsonConvert.SerializeObject(Data.Network, Formatting.Indented);
                    }

                    if (Data.Motherboard != null)
                    {
                        SSEMI.MotherboardData = JsonConvert.SerializeObject(Data.Motherboard, Formatting.Indented);
                    }
                }
            }
        }
    }
}

#endif