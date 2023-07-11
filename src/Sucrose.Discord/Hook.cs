using DiscordRPC;
using System.Globalization;
using SDMI = Sucrose.Discord.Manage.Internal;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSHC = Sucrose.Space.Helper.Command;

namespace Sucrose.Discord
{
    public class Hook
    {
        private string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private bool State => SMMI.DiscordSettingManager.GetSetting(SMC.State, true);

        public Hook()
        {
            SGMR.CultureInfo = new CultureInfo(Culture, true);

            SDMI.Client = new DiscordRpcClient(SMR.DiscordApplication)
            {
                //Logger = new DiscordRPC.Logging.ConsoleLogger()
                //{
                //    Level = DiscordRPC.Logging.LogLevel.Trace,
                //    Coloured = true
                //}
            };
        }

        public void Initialize()
        {
            if (State && SSHC.Work(SDMI.Name[0], SDMI.Name[1]))
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
                            Label = SGHDL.GetValue("DownloadButton"),
                            Url = SMR.DownloadWebsite
                        }
                    },
                    //Party = new Party()
                    //{
                    //    Max = 1,
                    //    Size = 1,
                    //    ID = Secrets.CreateFriendlySecret(SMR.Randomise),
                    //},
                    Assets = new Assets()
                    {
                        LargeImageKey = SGHDL.GetValue("LargestImage"),
                        SmallImageKey = SGHDL.GetValue("SmallestImage"),
                        LargeImageText = SDMI.Largest[SMR.Randomise.Next(SDMI.Largest.Count - 1)],
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