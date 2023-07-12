using DiscordRPC;
using DiscordRPC.Message;
using System.Globalization;
using Button = DiscordRPC.Button;
using SDMI = Sucrose.Discord.Manage.Internal;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSHP = Sucrose.Space.Helper.Processor;

namespace Sucrose.Discord
{
    internal class Hook
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

            SDMI.Client.OnReady += Client_OnReady;
            //SDMI.Client.OnClose += Client_OnClose;
            //SDMI.Client.OnError += Client_OnError;

            //SDMI.Client.OnConnectionFailed += Client_OnConnectionFailed;
            //SDMI.Client.OnConnectionEstablished += Client_OnConnectionEstablished;
        }

        public void Initialize()
        {
            if (State && SSHP.Work(SDMI.Name[0], SDMI.Name[1]))
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