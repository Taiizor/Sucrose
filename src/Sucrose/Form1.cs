using Grpc.Core;
using ReaLTaiizor.Forms;
using Sucrose.Grpc.Client.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;

namespace Sucrose
{
    public partial class Form1 : CrownForm
    {
        public Form1()
        {
            InitializeComponent();

            Internal.TrayIconManager.Start();

            GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
            {
                Greeter.BindService(new GreeterServerService()),
                Trayer.BindService(new TrayerServerService())
            });

            //MessageBox.Show(GeneralServerService.Host + "-" + GeneralServerService.Port);

            GeneralServerService.ServerInstance.Start();

            GeneralServerService.ChannelCreate();

            Greeter.GreeterClient client = new(GeneralServerService.ChannelInstance);
            GreetingResponse response = GreeterClientService.GetHello(client, "Taiizor");
            crownLabel1.Text = "Server response: " + response.Message;


            Trayer.TrayerClient client2 = new(GeneralServerService.ChannelInstance);
            TrayStateResponse response2 = TrayerClientService.GetState(client2);
            crownLabel2.Text = "Server response: " + response2.State;

            //GeneralServerService.ServerInstance.ShutdownAsync().Wait();
        }

        private void CrownButton1_Click(object sender, EventArgs e)
        {
            Trayer.TrayerClient client = new(GeneralServerService.ChannelInstance);
            TrayHideResponse response = TrayerClientService.GetHide(client);
        }

        private void CrownButton2_Click(object sender, EventArgs e)
        {
            Trayer.TrayerClient client = new(GeneralServerService.ChannelInstance);
            TrayShowResponse response = TrayerClientService.GetShow(client);
        }
    }
}
