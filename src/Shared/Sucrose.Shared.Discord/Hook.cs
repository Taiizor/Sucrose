using DiscordRPC;
using DiscordRPC.Message;
using Button = DiscordRPC.Button;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMH = Sucrose.Manager.Manage.Hook;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDMI = Sucrose.Shared.Discord.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;

namespace Sucrose.Shared.Discord
{
    internal class Hook : IDisposable
    {
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
            SMMI.HookSettingManager.SetSetting(SMMCH.DiscordState, true);
        }

        public void Disable()
        {
            SMMI.HookSettingManager.SetSetting(SMMCH.DiscordState, false);
        }

        public void SetPresence()
        {
            if (SSDMI.Client.IsInitialized)
            {
                SSDMI.Client.SetPresence(new RichPresence()
                {
                    Details = SRER.GetValue("Discord", "Details"),
                    //Timestamps = Timestamps.FromTimeSpan(60),
                    Timestamps = new Timestamps()
                    {
                        End = SSDMI.End,
                        Start = SSDMI.Start
                    },
                    State = SRER.GetValue("Discord", $"StatementText{SMMRG.Randomise.Next(49)}"),
                    Buttons = new Button[]
                    {
                        new()
                        {
                            Label = SRER.GetValue("Discord", "BrowseButton"),
                            Url = SMMRU.GitHubSucrose
                        },
                        new()
                        {
                            Label = SRER.GetValue("Discord", "DownloadButton"),
                            Url = SMMRU.MicrosoftStoreSucrose //SMMRU.GitHubSucroseRelease
                        }
                    },
                    //Party = new Party()
                    //{
                    //    Max = 1,
                    //    Size = 1,
                    //    ID = Secrets.CreateFriendlySecret(SMMRG.Randomise),
                    //},
                    Assets = new Assets()
                    {
                        LargeImageKey = SRER.GetValue("Discord", "LargestImage"),
                        LargeImageText = SRER.GetValue("Discord", $"LargestText{SMMRG.Randomise.Next(24)}"),
                        SmallImageText = SRER.GetValue("Discord", $"SmallestText{SMMRG.Randomise.Next(24)}"),
                        SmallImageKey = SRER.GetValue("Discord", $"SmallestImage{SMMRG.Randomise.Next(37)}")
                    }
                });
            }
        }

        public void AutoRefresh()
        {
            SSDMI.RefreshTimer.Tick += new EventHandler(RefreshTimer_Tick);
            SSDMI.RefreshTimer.Interval = new TimeSpan(0, 0, SMMH.DiscordDelay);
            SSDMI.RefreshTimer.Start();
        }

        public void ClearPresence()
        {
            SSDMI.Client.ClearPresence();
        }

        private void RefreshTimer_Tick(object sender, EventArgs e)
        {
            if (SMMH.DiscordRefresh)
            {
                SetPresence();
            }
        }

        private void InitializeTimer_Tick(object sender, EventArgs e)
        {
            if (SMMH.DiscordState && SSSHP.Work(SSDMI.Name[0], SSDMI.Name[1]))
            {
                if (!SSDMI.Client.IsInitialized)
                {
                    SSDMI.Client.Initialize();
                }

                if (SMMH.DiscordRefresh && !SSDMI.RefreshTimer.IsEnabled)
                {
                    AutoRefresh();
                }
                else if (!SMMH.DiscordRefresh && SSDMI.RefreshTimer.IsEnabled)
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