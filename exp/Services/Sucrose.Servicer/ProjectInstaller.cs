using System.Collections;
using System.ComponentModel;
using System.Configuration.Install;
using System.ServiceProcess;

namespace Sucrose.Servicer
{
    [RunInstaller(true)]
    public partial class ProjectInstaller : Installer
    {
        public ProjectInstaller()
        {
            InitializeComponent();

            // Servis için hesap türünü belirleyin
            serviceProcessInstaller1.Account = ServiceAccount.LocalSystem;

            // Hizmetin özelliklerini ayarlayın
            serviceInstaller1.DisplayName = "Sucrose Hizmetçisi";
            serviceInstaller1.StartType = ServiceStartMode.Automatic;
            serviceInstaller1.ServiceName = "Sucrose Servicer";

            // Installers koleksiyonuna ekleyin
            Installers.Add(serviceProcessInstaller1);
            Installers.Add(serviceInstaller1);
        }

        protected override void OnAfterInstall(IDictionary savedState)
        {
            using (ServiceController sc = new ServiceController(serviceInstaller1.ServiceName))
            {
                sc.Start();
            }

            base.OnAfterInstall(savedState);
        }

        protected override void OnBeforeInstall(IDictionary savedState)
        {
            string parameter = "MySource1\" \"MyLogFile1";
            Context.Parameters["assemblypath"] = "\"" + Context.Parameters["assemblypath"] + "\" \"" + parameter + "\"";

            base.OnBeforeInstall(savedState);
        }
    }
}
