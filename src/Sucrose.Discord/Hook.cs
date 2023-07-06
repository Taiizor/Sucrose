using DiscordRPC;
using System.Globalization;
using SDMI = Sucrose.Discord.Manage.Internal;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Discord
{
    public class Hook
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static bool State => SMMI.DiscordSettingManager.GetSetting(SMC.State, true);

        public Hook()
        {
            SGMR.CultureInfo = new CultureInfo(Culture, true);

            SDMI.Client = new DiscordRpcClient(SMR.DiscordApplication)
            {
                //Logger = new ConsoleLogger()
                //{
                //    Level = LogLevel.Trace,
                //    Coloured = true
                //}
            };
        }

        public void Initialize()
        {
            if (State)
            {
                SDMI.Client.Initialize();
            }
        }

        public void Update()
        {
            if (SDMI.Client.IsInitialized)
            {
                SDMI.Client.Invoke();
            }
        }

        public void Enable()
        {
            SMMI.DiscordSettingManager.SetSetting(SMC.State, true);
        }

        public void Disable()
        {
            SMMI.DiscordSettingManager.SetSetting(SMC.State, false);
        }

        public void SetPresence()
        {
            if (SDMI.Client.IsInitialized)
            {
                SDMI.Client.SetPresence(new RichPresence()
                {
                    Details = SGHDL.GetValue("Details"),
                    //Timestamps = Timestamps.FromTimeSpan(60),
                    Timestamps = new Timestamps()
                    {
                        Start = Timestamps.Now.Start,
                        End = Timestamps.Now.End
                    },
                    State = SDMI.Statement[SMR.Randomise.Next(SDMI.Statement.Count - 1)],
                    Buttons = new Button[]
                    {
                        new Button()
                        {
                            Label = SGHDL.GetValue("BrowseButton"),
                            Url = SMR.BrowseWebsite
                        },
                        new Button()
                        {
                            Label = SGHDL.GetValue("Download"),
                            Url = SMR.DownloadWebsite
                        }
                    },
                    Party = new Party()
                    {
                        ID = Secrets.CreateFriendlySecret(SMR.Randomise),
                        Max = 1,
                        Size = 1,
                    },
                    Assets = new Assets()
                    {
                        LargeImageKey = "logo-3840x3840",
                        LargeImageText = SDMI.Largest[SMR.Randomise.Next(SDMI.Largest.Count - 1)],
                        SmallImageKey = "logo-1024x1024",
                        SmallImageText = SDMI.Smallest[SMR.Randomise.Next(SDMI.Largest.Count - 1)]
                    }
                });
            }
        }

        public void ClearPresence()
        {
            SDMI.Client.ClearPresence();
        }

        public void Dispose()
        {
            if (SDMI.Client.IsInitialized)
            {
                SDMI.Client.Dispose();
            }
        }
    }
}