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
    internal class Hook : IDisposable
    {
        private int DiscordDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMC.DiscordDelay, 60), 60, 3600);

        private bool DiscordRefresh => SMMI.HookSettingManager.GetSetting(SMC.DiscordRefresh, true);

        private bool DiscordState => SMMI.HookSettingManager.GetSetting(SMC.DiscordState, true);

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
            SMMI.HookSettingManager.SetSetting(SMC.DiscordState, true);
        }

        public void Disable()
        {
            SMMI.HookSettingManager.SetSetting(SMC.DiscordState, false);
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
                            Url = SMR.DownloadStore //SMR.DownloadWebsite
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
            SSDMI.RefreshTimer.Tick += new EventHandler(RefreshTimer_Tick);
            SSDMI.RefreshTimer.Interval = new TimeSpan(0, 0, DiscordDelay);
            SSDMI.RefreshTimer.Start();
        }

        public void ClearPresence()
        {
            SSDMI.Client.ClearPresence();
        }

        private void RefreshTimer_Tick(object sender, EventArgs e)
        {
            if (DiscordRefresh)
            {
                SetPresence();
            }
        }

        private void InitializeTimer_Tick(object sender, EventArgs e)
        {
            if (DiscordState && SSSHP.Work(SSDMI.Name[0], SSDMI.Name[1]))
            {
                if (!SSDMI.Client.IsInitialized)
                {
                    SSDMI.Client.Initialize();
                }

                if (DiscordRefresh && !SSDMI.RefreshTimer.IsEnabled)
                {
                    AutoRefresh();
                }
                else if (!DiscordRefresh && SSDMI.RefreshTimer.IsEnabled)
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

        public void Dispose()
        {
            if (SSDMI.Client.IsInitialized)
            {
                SSDMI.Client.Dispose();
            }

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}