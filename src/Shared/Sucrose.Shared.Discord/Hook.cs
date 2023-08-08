using DiscordRPC;
using DiscordRPC.Message;
using Button = DiscordRPC.Button;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDMI = Sucrose.Shared.Discord.Manage.Internal;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Shared.Discord
{
    internal class Hook
    {
        private bool Refresh => SMMI.DiscordSettingManager.GetSetting(SMC.Refresh, true);

        private int Delay => SMMI.DiscordSettingManager.GetSettingStable(SMC.Delay, 60);

        private bool State => SMMI.DiscordSettingManager.GetSetting(SMC.State, true);

        public Hook()
        {
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
            SSDMI.InitializeTimer.Tick += new EventHandler(InitializeTimer_Tick);
            SSDMI.InitializeTimer.Interval = new TimeSpan(0, 0, 5);
            SSDMI.InitializeTimer.Start();
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
                    Details = SSRER.GetValue("Discord", "Details"),
                    //Timestamps = Timestamps.FromTimeSpan(60),
                    Timestamps = new Timestamps()
                    {
                        End = SSDMI.End,
                        Start = SSDMI.Start
                    },
                    State = SSRER.GetValue("Discord", $"StatementText{SMR.Randomise.Next(12)}"),
                    Buttons = new Button[]
                    {
                        new Button()
                        {
                            Label = SSRER.GetValue("Discord", "BrowseButton"),
                            Url = SMR.BrowseWebsite
                        },
                        new Button()
                        {
                            Label = SSRER.GetValue("Discord", "DownloadButton"),
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
                        LargeImageKey = SSRER.GetValue("Discord", "LargestImage"),
                        LargeImageText = SSRER.GetValue("Discord", $"LargestText{SMR.Randomise.Next(6)}"),
                        SmallImageText = SSRER.GetValue("Discord", $"SmallestText{SMR.Randomise.Next(6)}"),
                        SmallImageKey = SSRER.GetValue("Discord", $"SmallestImage{SMR.Randomise.Next(36)}")
                    }
                });
            }
        }

        public void AutoRefresh()
        {
            SSDMI.RefreshTimer.Interval = new TimeSpan(0, 0, SHS.Clamp(Delay, 60, 3600));
            SSDMI.RefreshTimer.Tick += new EventHandler(RefreshTimer_Tick);
            SSDMI.RefreshTimer.Start();
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

        private void RefreshTimer_Tick(object sender, EventArgs e)
        {
            if (Refresh)
            {
                SetPresence();
            }
        }

        private void InitializeTimer_Tick(object sender, EventArgs e)
        {
            if (State && SSSHP.Work(SSDMI.Name[0], SSDMI.Name[1]))
            {
                if (!SSDMI.Client.IsInitialized)
                {
                    SSDMI.Client.Initialize();
                }

                if (Refresh && !SSDMI.RefreshTimer.IsEnabled)
                {
                    AutoRefresh();
                }
                else if (!Refresh && SSDMI.RefreshTimer.IsEnabled)
                {
                    SSDMI.RefreshTimer.Stop();
                }

                if (SSDMI.Client.CurrentPresence == null)
                {
                    SetPresence();
                }
            }
            else
            {
                if (SSDMI.Client.IsInitialized)
                {
                    ClearPresence();
                }

                if (SSDMI.RefreshTimer.IsEnabled)
                {
                    SSDMI.RefreshTimer.Stop();
                }
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