using LibreHardwareMonitor.Hardware;

namespace Sucrose.Backgroundog.Extension
{
    internal class UpdateVisitor : IVisitor
    {
        public void VisitComputer(IComputer Computer)
        {
            Computer.Traverse(this);
        }

        public void VisitHardware(IHardware Hardware)
        {
            Hardware.Update();

            foreach (IHardware SubHardware in Hardware.SubHardware)
            {
                SubHardware.Accept(this);
            }
        }

        public void VisitSensor(ISensor Sensor) { }

        public void VisitParameter(IParameter Parameter) { }
    }
}