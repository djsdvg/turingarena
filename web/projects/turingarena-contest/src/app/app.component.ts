import { Component } from '@angular/core';
import { ContestQueryService } from './contest-query.service';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { SubmitDialogComponent } from './submit-dialog/submit-dialog.component';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent {
  constructor(
    private contestQueryService: ContestQueryService,
    private modal: NgbModal,
  ) { }

  contestQuery = this.contestQueryService.watch({}, {
    pollInterval: 10000,
  });

  selectedProblemName?: string = undefined;

  async openSubmitDialog() {
    const modalRef = this.modal.open(SubmitDialogComponent);
    const modal = modalRef.componentInstance as SubmitDialogComponent;

    modal.appComponent = this;
    modal.problemName = this.selectedProblemName;

    try {
      await modalRef.result;
    } catch (e) {
      // dismissed, do nothing
    }
  }
}
