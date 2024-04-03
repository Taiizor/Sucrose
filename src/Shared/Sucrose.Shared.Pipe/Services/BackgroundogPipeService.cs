#if LIVE_WEBVIEW || LIVE_CEFSHARP

using Newtonsoft.Json;
using SPEMREA = Sucrose.Pipe.Event.MessageReceivedEventArgs;
using SPIB = Sucrose.Pipe.Interface.Backgroundog;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Pipe.Services
{
    public static class BackgroundogPipeService
    {
        public static void Handler(SPEMREA e)
        {
            try
            {
                if (e != null && !string.IsNullOrEmpty(e.Message))
                {
                    SPIB Data = JsonConvert.DeserializeObject<SPIB>(e.Message);

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
            catch { }
        }
    }
}

#endif