using DiscordRPC;
using DiscordRPC.Message;
using System.Globalization;
using Button = DiscordRPC.Button;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDMI = Sucrose.Shared.Discord.Manage.Internal;
using SSHP = Sucrose.Space.Helper.Processor;

namespace Sucrose.Shared.Discord
{
    internal class Hook
    {
        private string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private bool Refresh => SMMI.DiscordSettingManager.GetSetting(SMC.Refresh, true);

        private int Delay => SMMI.DiscordSettingManager.GetSettingStable(SMC.Delay, 60);

        private bool State => SMMI.DiscordSettingManager.GetSetting(SMC.State, true);

        public Hook()
        {
            SGMR.CultureInfo = new CultureInfo(Culture, true);

            SSDMI.Client = new DiscordRpcClient(SMR.DiscordApplication)
            {
                //Logger = new DiscordRPC.Logging.ConsoleLogger()
                //{
                //    Level = DiscordRPC.Logging.LogLevel.Trace,
                //    Coloured = true
                //}
            };

            SSDMI.Client.OnReady += Client_OnReady;
            //SSDMI.Client.OnClose += Client_OnClose;
            //SSDMI.Client.OnError += Client_OnError;

            //SSDMI.Client.OnConnectionFailed += Client_OnConnectionFailed;
            //SSDMI.Client.OnConnectionEstablished += Client_OnConnectionEstablished;
        }

        public void Initialize()
        {
            if (State && SSHP.Work(SSDMI.Name[0], SSDMI.Name[1]))
            {
                SSDMI.Client.Initialize();

                AutoRefresh();
            }
        }

        public void Update()
        {
            if (SSDMI.Client.IsInitialized)
            {
                SSDMI.Client.Invoke();
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
            if (SSDMI.Client.IsInitialized)
            {
                SSDMI.Client.SetPresence(new RichPresence()
                {
                    Details = SGHDL.GetValue("Details"),
                    //Timestamps = Timestamps.FromTimeSpan(60),
                    Timestamps = new Timestamps()
                    {
                        End = SSDMI.End,
                        Start = SSDMI.Start
                    },
                    State = SSDMI.Statement[SMR.Randomise.Next(SSDMI.Statement.Count - 1)],
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
                        LargeImageText = SSDMI.LargestText[SMR.Randomise.Next(SSDMI.LargestText.Count - 1)],
                        SmallImageText = SSDMI.SmallestText[SMR.Randomise.Next(SSDMI.LargestText.Count - 1)],
                        SmallImageKey = SSDMI.SmallestImage[SMR.Randomise.Next(SSDMI.SmallestImage.Count - 1)]
                    }
                });
            }
        }

        public void AutoRefresh()
        {
            SSDMI.Timer.Interval = new TimeSpan(0, 0, SHS.Clamp(Delay, 60, 3600));
            SSDMI.Timer.Tick += new EventHandler(Timer_Tick);
            SSDMI.Timer.Start();
        }

        public void ClearPresence()
        {
            SSDMI.Client.ClearPresence();
        }

        public void Dispose()
        {
            if (SSDMI.Client.IsInitialized)
            {
                SSDMI.Client.Dispose();
            }
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            if (Refresh)
            {
                SGMR.CultureInfo = new CultureInfo(Culture, true);

                SetPresence();
            }
        }

        private void Client_OnConnectionFailed(object sender, ConnectionFailedMessage args)
        {
            Console.WriteLine("DiscordRPC connection error.");
        }

        private void Client_OnConnectionEstablished(object sender, ConnectionEstablishedMessage args)
        {
            Console.WriteLine("DiscordRPC connection established.");
            //SetPresence();
        }

        private void Client_OnReady(object sender, ReadyMessage args)
        {
            SetPresence();
        }

        private void Client_OnClose(object sender, CloseMessage args)
        {
            Console.WriteLine("DiscordRPC connection closed. Code: " + args.Code + ", Reason: " + args.Reason);
        }

        private void Client_OnError(object sender, ErrorMessage args)
        {
            Console.WriteLine("DiscordRPC error: " + args.Message);
        }
    }
}